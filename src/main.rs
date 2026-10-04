mod cli;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use feros_flasher::repository;
use feros_flasher::{
    error::Result,
    host::{self, human_size, validate_selection},
    image, target,
};

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
        /// Display help for the channel command.
        #[arg(short = 'c', long, conflicts_with_all = ["list", "flash", "version"])]
        channel: bool,

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

    /// Channel for repository devices.
    Channel {
        /// Add a channel interactively.
        #[arg(long, conflicts_with = "list")]
        add: bool,

        /// List available channels.
        #[arg(long)]
        list: bool,

        /// Set the default repository by its list number.
        #[arg(long, value_name = "NUMBER", conflicts_with_all = ["add", "list"])]
        select: Option<usize>,

        /// Delete a repository by its list number.
        #[arg(long, value_name = "NUMBER",conflicts_with_all = ["add", "list", "select"])]
        delete: Option<usize>,
    },

    /// List eligible physical devices without modifying them.
    List,

    /// Write and verify a FeROS repository image.
    Flash {
        /// Canonical FeROS repository identifier.
        #[arg(long)]
        target: String,

        /// Path to the prepared repository image.
        #[arg(long)]
        image: PathBuf,

        /// Optional whole-device path, such as /dev/disk4.
        #[arg(long)]
        device: Option<String>,
    },
}

#[derive(Clone, Copy)]
enum HelpTopic {
    Channel,
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
            channel,
            list,
            flash,
            version,
        }) => {
            let topic = if channel {
                Some(HelpTopic::Channel)
            } else if list {
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
        Some(Commands::Channel {
            add,
            list,
            select,
            delete,
        }) => {
            main_header()?;

            if add {
                println!("\nAdding Device Support Repository\n");

                let url = cli::prompt("Repository URL")?;
                let name = cli::prompt("Repository name [leave blank to use default name]")?;
                let is_default = loop {
                    let answer = cli::prompt("Set as default? [y/N]")?;

                    match answer.to_ascii_lowercase().as_str() {
                        "" | "n" | "no" => break false,
                        "y" | "yes" => break true,
                        _ => println!("Please enter yes or no."),
                    }
                };

                let added_name = repository::add(&url, Some(&name), is_default)?;

                println!("Repository added: {added_name}");
            }

            if list {
                let repositories = repository::available()?;

                if repositories.is_empty() {
                    println!("\nNo repositories configured.");
                } else {
                    println!("\nConfigured repositories:");

                    for (index, entry) in repositories.iter().enumerate() {
                        let marker = if entry.is_default { "*" } else { " " };
                        let category = if entry.official {
                            "official"
                        } else {
                            "unofficial"
                        };

                        println!("  {marker} {}) {} [{category}]", index + 1, entry.name);
                        println!("       {}", entry.url);
                    }
                }
            }

            if let Some(number) = select {
                let name = repository::select(number)?;
                println!("\nDefault repository changed to: {name}");
            }

            if let Some(number) = delete {
                let name = repository::delete(number)?;
                println!("Repository deleted: {name}");
            }
        }
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
            let selected = cli::select_device(&backend.eligible_devices()?, device.as_deref())?;
            validate_selection(&selected, &selected, image.size())?;
            cli::confirm(&target, &image, &selected)?;
            backend.flash(&image, &selected, &mut |event| {
                cli::show_progress(event, &selected)
            })?;
        }
    }

    Ok(())
}

fn main_header() -> Result<()> {
    println!("{PRODUCT_NAME}");
    println!("{PRODUCT_DESCRIPTION}\n");

    println!("Repository:");
    match repository::current()? {
        Some(current) => println!("Current: {}\n", current.name),
        None => println!("Current: none selected\n"),
    }

    Ok(())
}

fn help_header() {
    println!("{PRODUCT_NAME}\n");
    println!("{PRODUCT_DESCRIPTION}\n");
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

fn print_help(topic: Option<HelpTopic>) {
    help_header();
    println!();

    match topic {
        None => {
            println!("Usage:");
            println!("  flasher <command> [options]\n");
            println!("Commands:");
            println!("  channel   Channel list and management operations");
            println!("  list      List eligible physical devices");
            println!("  flash     Write and verify a FeROS repository image");
            println!("  help      Display general or command-specific help");
            println!("  version   Display build and version information\n");
            println!("Command help:");
            println!("  flasher help --channel");
            println!("  flasher help --list");
            println!("  flasher help --flash");
            println!("  flasher help --version");
        }
        Some(HelpTopic::Channel) => {
            println!("Manage device support repositories.\n");
            println!("Usage:");
            println!("  flasher channel                  Display the default repository");
            println!("  flasher channel --list           List configured repositories");
            println!("  flasher channel --add            Add a repository interactively");
            println!("  flasher channel --select <INDEX> Set the default repository");
            println!("  flasher channel --delete <INDEX> Delete a non-default repository");
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
    help_header();
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
