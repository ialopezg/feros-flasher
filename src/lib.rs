//! Shared image, target, and physical-media operations for FeROS Flasher.
//!
//! Frontends own device selection, explicit user confirmation, and presentation.
//! The macOS privilege mechanism still uses sudo and requires a terminal.

pub mod error;
pub mod host;
pub mod image;
pub mod target;
