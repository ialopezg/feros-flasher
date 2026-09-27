mod error;
mod host;
mod image;
mod target;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{error::Result, host::human_size};

#[derive(Debug, Parser)]
#[command(name = "feros-flasher")]
#[command(about = "Install and verify FeROS images on physical media")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List eligible physical devices without modifying them.
    List,

    /// Write and verify a FeROS target image.
    Flash {
        /// Canonical FeROS target identifier.
        #[arg(long)]
        target: String,

        /// Path to the prepared target image.
        #[arg(long)]
        image: PathBuf,

        /// Optional whole-device path, such as /dev/disk4.
        #[arg(long)]
        device: Option<String>,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FeROS Flasher: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let backend = host::current()?;

    match cli.command {
        Commands::List => {
            let devices = backend.eligible_devices()?;
            if devices.is_empty() {
                println!("FeROS Flasher: no eligible removable physical disks were found.");
                return Ok(());
            }

            println!("FeROS Flasher: eligible physical disks:\n");
            for device in devices {
                println!(
                    "  {} — {} — {} — {}",
                    device.device_path,
                    device.model,
                    human_size(device.size),
                    device.protocol
                );
            }
        }
        Commands::Flash {
            target: target_id,
            image: image_path,
            device,
        } => {
            let target = target::resolve(&target_id)?;
            let image = image::Image::inspect(&image_path, &target)?;
            backend.flash(&target, &image, device.as_deref())?;
        }
    }

    Ok(())
}
