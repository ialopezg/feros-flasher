use std::{
    io::{self, Write},
    process::{Command, Output},
};

use plist::{Dictionary, Value};

use crate::{
    error::{FlasherError, Result},
    host::{human_size, Device, MediaBackend},
    image::Image,
    target::Target,
};

struct Backend {
    startup_disk: String,
}

impl Backend {
    fn new() -> Result<Self> {
        reject_root_execution()?;
        Ok(Self {
            startup_disk: startup_disk_identifier()?,
        })
    }

    fn discover_physical_disks(&self) -> Result<Vec<Device>> {
        let listing = diskutil_plist(&["list", "physical"])?;
        let identifiers = whole_disk_identifiers(&listing)?;

        let mut devices = Vec::new();
        for identifier in identifiers {
            let info = diskutil_plist(&["info", &format!("/dev/{identifier}")])?;
            if let Some(device) = self.device_from_info(&info)? {
                devices.push(device);
            }
        }

        devices.sort_by(|left, right| left.identifier.cmp(&right.identifier));
        Ok(devices)
    }

    fn device_from_info(&self, info: &Dictionary) -> Result<Option<Device>> {
        let Some(identifier) = string_value(info, "DeviceIdentifier") else {
            return Ok(None);
        };

        if !is_whole_disk_identifier(&identifier)
            || !bool_value(info, "WholeDisk")
                .or_else(|| bool_value(info, "Whole"))
                .unwrap_or(false)
            || identifier == self.startup_disk
        {
            return Ok(None);
        }

        let physical = string_value(info, "VirtualOrPhysical")
            .map(|value| value == "Physical")
            .unwrap_or(true);
        let internal = bool_value(info, "Internal").unwrap_or(true);
        let removable = bool_value(info, "Removable")
            .or_else(|| bool_value(info, "RemovableMedia"))
            .unwrap_or(false);
        let writable = bool_value(info, "Writable")
            .or_else(|| bool_value(info, "MediaWritable"))
            .unwrap_or_else(|| !bool_value(info, "MediaReadOnly").unwrap_or(true));

        if !physical || !writable || (internal && !removable) {
            return Ok(None);
        }

        let size = integer_value(info, "TotalSize").unwrap_or(0);
        if size == 0 {
            return Ok(None);
        }

        let model = string_value(info, "MediaName")
            .or_else(|| string_value(info, "DeviceModel"))
            .unwrap_or_else(|| "Unknown media".to_owned());
        let protocol =
            string_value(info, "BusProtocol").unwrap_or_else(|| "Unknown protocol".to_owned());

        Ok(Some(Device {
            device_path: format!("/dev/{identifier}"),
            raw_device_path: format!("/dev/r{identifier}"),
            identifier,
            model,
            protocol,
            size,
            internal,
            removable,
        }))
    }

    fn select_device(&self, devices: &[Device], requested: Option<&str>) -> Result<Device> {
        if devices.is_empty() {
            return Err(FlasherError::message(
                "no eligible removable physical disks were found",
            ));
        }

        if let Some(requested) = requested {
            return devices
                .iter()
                .find(|device| device.device_path == requested || device.identifier == requested)
                .cloned()
                .ok_or_else(|| {
                    FlasherError::message(format!("requested device is not eligible: {requested}"))
                });
        }

        print_devices(devices);
        print!("Select [0-{}]: ", devices.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection = input
            .trim()
            .parse::<usize>()
            .map_err(|_| FlasherError::message("invalid device selection"))?;

        if selection == 0 {
            return Err(FlasherError::message(
                "operation cancelled; no storage device was modified",
            ));
        }

        devices
            .get(selection - 1)
            .cloned()
            .ok_or_else(|| FlasherError::message("invalid device selection"))
    }

    fn refresh_selected_device(&self, selected: &Device) -> Result<Device> {
        let current = self.discover_physical_disks()?;
        current
            .into_iter()
            .find(|device| device.identifier == selected.identifier)
            .ok_or_else(|| {
                FlasherError::message(format!(
                    "selected device is no longer eligible: {}",
                    selected.device_path
                ))
            })
    }

    fn confirm(&self, target: &Target, image: &Image, device: &Device) -> Result<()> {
        println!("\nFeROS Flasher: destructive media operation\n");
        println!("  Target:        {} ({})", target.display_name, target.id);
        println!("  Device:        {}", device.device_path);
        println!("  Model:         {}", device.model);
        println!("  Protocol:      {}", device.protocol);
        println!("  Capacity:      {}", human_size(device.size));
        println!("  Internal:      {}", device.internal);
        println!("  Removable:     {}", device.removable);
        println!("  Image:         {}", image.path().display());
        println!("  Write size:    {}", human_size(image.size()));
        println!("  Image SHA-256: {}", image.sha256());
        println!(
            "\nExisting partition and filesystem metadata in the written region will be overwritten."
        );
        println!("This operation cannot be undone.\n");

        let phrase = format!("WRITE {}", device.device_path);
        print!("Type '{phrase}' to continue: ");
        io::stdout().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        if confirmation.trim() != phrase {
            return Err(FlasherError::message(
                "operation aborted: required user confirmation was not provided; no storage device was modified",
            ));
        }

        Ok(())
    }

    fn write_and_verify(&self, image: &Image, device: &Device) -> Result<()> {
        println!("\nFeROS Flasher: unmounting {}...", device.device_path);
        run_checked(Command::new("diskutil").args(["unmountDisk", &device.device_path]))?;

        println!(
            "FeROS Flasher: writing {} to {}...",
            human_size(image.size()),
            device.raw_device_path
        );
        run_checked(
            Command::new("sudo")
                .arg("dd")
                .arg(format!("if={}", image.path().to_string_lossy()))
                .arg(format!("of={}", device.raw_device_path))
                .arg("bs=1048576"),
        )?;
        run_checked(&mut Command::new("sync"))?;

        println!("FeROS Flasher: verifying written bytes...");
        run_checked(
            Command::new("sudo")
                .arg("cmp")
                .arg("-n")
                .arg(image.size().to_string())
                .arg(image.path())
                .arg(&device.raw_device_path),
        )?;

        println!("FeROS Flasher: ejecting {}...", device.device_path);
        run_checked(Command::new("diskutil").args(["eject", &device.device_path]))?;

        Ok(())
    }
}

fn whole_disk_identifiers(listing: &Dictionary) -> Result<Vec<String>> {
    if let Some(disks) = listing.get("WholeDisks").and_then(Value::as_array) {
        let identifiers = disks
            .iter()
            .filter_map(Value::as_string)
            .map(str::to_owned)
            .collect::<Vec<_>>();

        if !identifiers.is_empty() {
            return Ok(identifiers);
        }
    }

    let entries = listing
        .get("AllDisksAndPartitions")
        .and_then(Value::as_array)
        .ok_or_else(|| FlasherError::message("diskutil did not return a physical disk list"))?;

    let identifiers = entries
        .iter()
        .filter_map(Value::as_dictionary)
        .filter_map(|entry| string_value(entry, "DeviceIdentifier"))
        .collect::<Vec<_>>();

    if identifiers.is_empty() {
        return Err(FlasherError::message(
            "diskutil returned an empty physical disk list",
        ));
    }

    Ok(identifiers)
}

impl MediaBackend for Backend {
    fn eligible_devices(&self) -> Result<Vec<Device>> {
        self.discover_physical_disks()
    }

    fn flash(&self, target: &Target, image: &Image, requested: Option<&str>) -> Result<()> {
        let selected = self.select_device(&self.discover_physical_disks()?, requested)?;
        if image.size() > selected.size {
            return Err(FlasherError::message(format!(
                "image size {} exceeds device capacity {}",
                human_size(image.size()),
                human_size(selected.size)
            )));
        }

        let selected = self.refresh_selected_device(&selected)?;
        self.confirm(target, image, &selected)?;

        let selected = self.refresh_selected_device(&selected)?;
        self.write_and_verify(image, &selected)?;
        println!("\nFeROS Flasher: target media written and verified successfully.");
        Ok(())
    }
}

pub(super) fn backend() -> Result<Box<dyn MediaBackend>> {
    Ok(Box::new(Backend::new()?))
}

fn startup_disk_identifier() -> Result<String> {
    let info = diskutil_plist(&["info", "/"])?;
    string_value(&info, "ParentWholeDisk")
        .or_else(|| string_value(&info, "DeviceIdentifier"))
        .ok_or_else(|| FlasherError::message("cannot determine the macOS startup disk"))
}

fn reject_root_execution() -> Result<()> {
    let output = run_output(Command::new("id").arg("-u"))?;
    let effective_user = String::from_utf8_lossy(&output.stdout);
    if effective_user.trim() == "0" {
        return Err(FlasherError::message(
            "run FeROS Flasher as a normal user; privilege is requested only after confirmation",
        ));
    }
    Ok(())
}

fn diskutil_plist(arguments: &[&str]) -> Result<Dictionary> {
    if arguments.is_empty() {
        return Err(FlasherError::message("diskutil operation is required"));
    }

    let mut command = Command::new("diskutil");
    command.arg(arguments[0]).arg("-plist");
    command.args(&arguments[1..]);
    let output = run_output(&mut command)?;
    let value = Value::from_reader_xml(output.stdout.as_slice())?;
    value
        .into_dictionary()
        .ok_or_else(|| FlasherError::message("diskutil returned a non-dictionary plist"))
}

fn run_output(command: &mut Command) -> Result<Output> {
    let rendered = format!("{command:?}");
    let output = command.output()?;
    if output.status.success() {
        return Ok(output);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        Err(FlasherError::message(format!("command failed: {rendered}")))
    } else {
        Err(FlasherError::message(stderr))
    }
}

fn run_checked(command: &mut Command) -> Result<()> {
    let rendered = format!("{command:?}");
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(FlasherError::message(format!("command failed: {rendered}")))
    }
}

fn print_devices(devices: &[Device]) {
    println!("FeROS Flasher: eligible physical disks:\n");
    for (index, device) in devices.iter().enumerate() {
        println!(
            "  {}) {} — {} — {} — {}",
            index + 1,
            device.device_path,
            device.model,
            human_size(device.size),
            device.protocol
        );
    }
    println!("  0) Cancel\n");
}

fn is_whole_disk_identifier(identifier: &str) -> bool {
    identifier
        .strip_prefix("disk")
        .map(|suffix| !suffix.is_empty() && suffix.bytes().all(|value| value.is_ascii_digit()))
        .unwrap_or(false)
}

fn string_value(dictionary: &Dictionary, key: &str) -> Option<String> {
    dictionary
        .get(key)
        .and_then(Value::as_string)
        .map(str::to_owned)
}

fn bool_value(dictionary: &Dictionary, key: &str) -> Option<bool> {
    dictionary.get(key).and_then(Value::as_boolean)
}

fn integer_value(dictionary: &Dictionary, key: &str) -> Option<u64> {
    dictionary.get(key).and_then(Value::as_unsigned_integer)
}
