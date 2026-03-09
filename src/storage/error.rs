use thiserror::Error;

use crate::installer::types::PackageIdError;

/// The errors that occur when reading or saving the register file.
#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Cannot read or write installed packages file")]
    IOError(#[from] std::io::Error),

    #[error("Cannot parse installed packages file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot serialize installed packages")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Cannot add package with invalid package id")]
    PackageIdError(#[from] PackageIdError),
}

pub(super) type Result<T> = std::result::Result<T, RegisterError>;
