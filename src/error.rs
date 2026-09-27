use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlasherError {
    #[error("{0}")]
    Message(String),

    #[error("I/O failure: {0}")]
    Io(#[from] io::Error),

    #[error("invalid property-list data: {0}")]
    Plist(#[from] plist::Error),
}

pub type Result<T> = std::result::Result<T, FlasherError>;

impl FlasherError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
