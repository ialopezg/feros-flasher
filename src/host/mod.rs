#[cfg(target_os = "macos")]
mod darwin;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use crate::{
    error::{FlasherError, Result},
    image::Image,
};

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Stage notifications, not byte-level progress or cancellation callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashEvent {
    Unmounting,
    Writing { bytes: u64 },
    Verifying,
    Ejecting,
    Completed,
}

pub trait MediaBackend {
    fn eligible_devices(&self) -> Result<Vec<Device>>;
    /// Writes and verifies an explicitly selected device.
    ///
    /// The caller must obtain user confirmation before calling this method.
    /// Implementations must revalidate eligibility, the selected device's
    /// reported metadata, and capacity before modifying the medium.
    fn flash(
        &self,
        image: &Image,
        device: &Device,
        observer: &mut dyn FnMut(FlashEvent),
    ) -> Result<()>;
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

/// Checks the available device metadata; it is not a stable hardware identifier
/// and cannot distinguish two devices reporting identical values.
pub fn validate_selection(selected: &Device, current: &Device, image_size: u64) -> Result<()> {
    if selected != current {
        return Err(FlasherError::message(format!(
            "selected device changed; select and confirm it again: {}",
            selected.device_path
        )));
    }
    if image_size > current.size {
        return Err(FlasherError::message(format!(
            "image size {} exceeds device capacity {}",
            human_size(image_size),
            human_size(current.size)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> Device {
        Device {
            identifier: "disk4".into(),
            device_path: "/dev/disk4".into(),
            raw_device_path: "/dev/rdisk4".into(),
            model: "Test media".into(),
            protocol: "USB".into(),
            size: 4096,
            internal: false,
            removable: true,
        }
    }

    #[test]
    fn rejects_reused_path_with_changed_media() {
        let selected = device();
        let mut current = selected.clone();
        current.model = "Different media".into();
        assert!(validate_selection(&selected, &current, 1024).is_err());
    }

    #[test]
    fn rejects_capacity_changes_after_selection() {
        let selected = device();
        let mut current = selected.clone();
        current.size = 512;
        assert!(validate_selection(&selected, &current, 1024).is_err());
    }

    #[test]
    fn checks_capacity_boundary() {
        let selected = device();
        assert!(validate_selection(&selected, &selected, 4096).is_ok());
        assert!(validate_selection(&selected, &selected, 4097).is_err());
    }
}
