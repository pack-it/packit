// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::installer::types::PackageIdError;

/// The errors that occur when reading or saving the register file.
#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("The Register.toml file does not exist or is empty")]
    RegisterDoesNotExist,

    #[error("Cannot add package with invalid package id")]
    PackageIdError(#[from] PackageIdError),

    #[error("Cannot read or write installed packages file")]
    IOError(#[from] std::io::Error),

    #[error("Cannot parse installed packages file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot serialize installed packages")]
    SerializeError(#[from] toml::ser::Error),
}

pub(super) type Result<T> = std::result::Result<T, RegisterError>;
