use crate::{
    error::{FlasherError, Result},
    host::{Device, FlashEvent, MediaBackend},
    image::Image,
};

struct Backend;

impl MediaBackend for Backend {
    fn eligible_devices(&self) -> Result<Vec<Device>> {
        Err(unsupported())
    }

    fn flash(
        &self,
        _image: &Image,
        _device: &Device,
        _observer: &mut dyn FnMut(FlashEvent),
    ) -> Result<()> {
        Err(unsupported())
    }
}

pub(super) fn backend() -> Result<Box<dyn MediaBackend>> {
    Ok(Box::new(Backend))
}

fn unsupported() -> FlasherError {
    FlasherError::message("the Linux media backend is not supported yet")
}
