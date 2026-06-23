// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{installer::types::PackageIdError, utils::ioerror};

/// The errors that occur when reading or saving the register file.
#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("The register file does not exist or is empty")]
    RegisterDoesNotExist,

    #[error("Cannot add package with invalid package id")]
    PackageIdError(#[from] PackageIdError),

    #[error("Cannot read or write register file")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot parse register file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot serialize register")]
    SerializeError(#[from] toml::ser::Error),
}

pub(super) type Result<T> = std::result::Result<T, RegisterError>;
