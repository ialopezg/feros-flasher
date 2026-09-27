#[cfg(target_os = "macos")]
mod darwin;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use crate::error::FlasherError;

use crate::{error::Result, image::Image, target::Target};

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct Device {
    pub identifier: String,
    pub device_path: String,
    pub raw_device_path: String,
    pub model: String,
    pub protocol: String,
    pub size: u64,
    pub internal: bool,
    pub removable: bool,
}

pub trait MediaBackend {
    fn eligible_devices(&self) -> Result<Vec<Device>>;
    fn flash(&self, target: &Target, image: &Image, device: Option<&str>) -> Result<()>;
}

pub fn current() -> Result<Box<dyn MediaBackend>> {
    #[cfg(target_os = "macos")]
    {
        darwin::backend()
    }

    #[cfg(target_os = "linux")]
    {
        linux::backend()
    }

    #[cfg(target_os = "windows")]
    {
        windows::backend()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(FlasherError::message(format!(
            "host operating system is not supported yet: {}",
            std::env::consts::OS
        )))
    }
}

pub fn human_size(size: u64) -> String {
    let mut value = size as f64;
    for unit in ["B", "KiB", "MiB", "GiB", "TiB"] {
        if value < 1024.0 || unit == "TiB" {
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    unreachable!()
}
