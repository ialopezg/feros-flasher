mod error;
mod host;
mod image;
mod target;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{error::Result, host::human_size};

const PRODUCT_NAME: &str = "FeROS Flasher";
const PRODUCT_DESCRIPTION: &str = "Safe physical-media installation and recovery for FeROS";

#[derive(Debug, Parser)]
#[command(name = "flasher")]
#[command(about = PRODUCT_DESCRIPTION)]
#[command(disable_help_flag = true)]
#[command(disable_help_subcommand = true)]
#[command(disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Display general or command-specific help.
    Help {
        /// Display help for the list command.
        #[arg(short = 'l', long, conflicts_with_all = ["flash", "version"])]
        list: bool,

        /// Display help for the flash command.
        #[arg(short = 'f', long, conflicts_with_all = ["list", "version"])]
        flash: bool,

        /// Display help for the version command.
        #[arg(short = 'v', long, conflicts_with_all = ["list", "flash"])]
        version: bool,
    },

    /// Display build and version information.
    Version,

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

#[derive(Clone, Copy)]
enum HelpTopic {
    List,
    Flash,
    Version,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FeROS Flasher: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => print_help(None),
        Some(Commands::Help {
            list,
            flash,
            version,
        }) => {
            let topic = if list {
                Some(HelpTopic::List)
            } else if flash {
                Some(HelpTopic::Flash)
            } else if version {
                Some(HelpTopic::Version)
            } else {
                None
            };

            print_help(topic);
        }
        Some(Commands::Version) => print_version(),
        Some(Commands::List) => {
            let backend = host::current()?;
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
        Some(Commands::Flash {
            target: target_id,
            image: image_path,
            device,
        }) => {
            let backend = host::current()?;
            let target = target::resolve(&target_id)?;
            let image = image::Image::inspect(&image_path, &target)?;
            backend.flash(&target, &image, device.as_deref())?;
        }
    }

    Ok(())
}

fn print_header() {
    println!("{PRODUCT_NAME}\n");
    println!("{PRODUCT_DESCRIPTION}\n");
}

fn print_help(topic: Option<HelpTopic>) {
    print_header();

    match topic {
        None => {
            println!("Usage:");
            println!("  flasher <command> [options]\n");
            println!("Commands:");
            println!("  list      List eligible physical devices");
            println!("  flash     Write and verify a FeROS target image");
            println!("  help      Display general or command-specific help");
            println!("  version   Display build and version information\n");
            println!("Command help:");
            println!("  flasher help --list");
            println!("  flasher help --flash");
            println!("  flasher help --version");
        }
        Some(HelpTopic::List) => {
            println!("List eligible physical devices without modifying them.\n");
            println!("Usage:");
            println!("  flasher list");
        }
        Some(HelpTopic::Flash) => {
            println!("Write and verify a FeROS target image.\n");
            println!("Usage:");
            println!("  flasher flash \\");
            println!("    --target <TARGET> \\");
            println!("    --image <IMAGE> \\");
            println!("    [--device <DEVICE>]\n");
            println!("Options:");
            println!("  --target <TARGET>   Canonical FeROS target identifier");
            println!("  --image <IMAGE>     Path to the prepared target image");
            println!("  --device <DEVICE>   Optional whole-device path");
        }
        Some(HelpTopic::Version) => {
            println!("Display build and version information.\n");
            println!("Usage:");
            println!("  flasher version");
        }
    }
}

fn print_version() {
    print_header();
    println!("Author: {}", env!("CARGO_PKG_AUTHORS"));
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!(
        "Commit: {}",
        option_env!("FEROS_BUILD_COMMIT").unwrap_or("none")
    );
    println!(
        "Built: {}",
        option_env!("FEROS_BUILD_TIMESTAMP").unwrap_or("unknown")
    );
    println!(
        "Processor: {} ({})",
        processor_name(),
        operating_system_name()
    );
}

fn processor_name() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        architecture => architecture,
    }
}

fn operating_system_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        operating_system => operating_system,
    }
}
