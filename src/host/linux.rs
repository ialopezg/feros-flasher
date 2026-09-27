use crate::{
    error::{FlasherError, Result},
    host::{Device, MediaBackend},
    image::Image,
    target::Target,
};

struct Backend;

impl MediaBackend for Backend {
    fn eligible_devices(&self) -> Result<Vec<Device>> {
        Err(unsupported())
    }

    fn flash(&self, _target: &Target, _image: &Image, _device: Option<&str>) -> Result<()> {
        Err(unsupported())
    }
}

pub(super) fn backend() -> Result<Box<dyn MediaBackend>> {
    Ok(Box::new(Backend))
}

fn unsupported() -> FlasherError {
    FlasherError::message("the Linux media backend is not supported yet")
}
