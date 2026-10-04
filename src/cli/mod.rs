use std::io::{self, Write};

use feros_flasher::{
    error::{FlasherError, Result},
    host::{human_size, Device, FlashEvent},
    image::Image,
    target::Target,
};

pub fn select_device(devices: &[Device], requested: Option<&str>) -> Result<Device> {
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

pub fn confirm(target: &Target, image: &Image, device: &Device) -> Result<()> {
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

pub fn show_progress(event: FlashEvent, device: &Device) {
    match event {
        FlashEvent::Unmounting => println!("\nFeROS Flasher: unmounting {}...", device.device_path),
        FlashEvent::Writing { bytes } => println!(
            "FeROS Flasher: writing {} to {}...",
            human_size(bytes),
            device.raw_device_path
        ),
        FlashEvent::Verifying => println!("FeROS Flasher: verifying written bytes..."),
        FlashEvent::Ejecting => println!("FeROS Flasher: ejecting {}...", device.device_path),
        FlashEvent::Completed => {
            println!("\nFeROS Flasher: target media written and verified successfully.")
        }
    }
}
