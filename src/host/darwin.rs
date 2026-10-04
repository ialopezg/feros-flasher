use std::process::{Command, Output};

use plist::{Dictionary, Value};

use crate::{
    error::{FlasherError, Result},
    host::{validate_selection, Device, FlashEvent, MediaBackend},
    image::Image,
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

    fn write_and_verify(
        &self,
        image: &Image,
        device: &Device,
        observer: &mut dyn FnMut(FlashEvent),
    ) -> Result<()> {
        observer(FlashEvent::Unmounting);
        run_checked(Command::new("diskutil").args(["unmountDisk", &device.device_path]))?;

        // Recheck after unmounting, before starting the writer.
        let current = self.refresh_selected_device(device)?;
        validate_selection(device, &current, image.size())?;
        observer(FlashEvent::Writing {
            bytes: image.size(),
        });
        run_checked(
            Command::new("sudo")
                .arg("dd")
                .arg(format!("if={}", image.path().to_string_lossy()))
                .arg(format!("of={}", device.raw_device_path))
                .arg("bs=1048576"),
        )?;
        run_checked(&mut Command::new("sync"))?;

        observer(FlashEvent::Verifying);
        run_checked(
            Command::new("sudo")
                .arg("cmp")
                .arg("-n")
                .arg(image.size().to_string())
                .arg(image.path())
                .arg(&device.raw_device_path),
        )?;

        observer(FlashEvent::Ejecting);
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

    fn flash(
        &self,
        image: &Image,
        selected: &Device,
        observer: &mut dyn FnMut(FlashEvent),
    ) -> Result<()> {
        let current = self.refresh_selected_device(selected)?;
        validate_selection(selected, &current, image.size())?;
        self.write_and_verify(image, &current, observer)?;
        observer(FlashEvent::Completed);
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
    // Capture child diagnostics instead of writing them directly to a frontend.
    // sudo may still use /dev/tty for authentication; GUI authorization is separate work.
    run_output(command).map(|_| ())
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
