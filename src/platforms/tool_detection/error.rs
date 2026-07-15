// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{installer::types::VersionError, utils::ioerror};

#[derive(Error, Debug)]
pub enum ToolDetectionError {
    #[error("Error while parsing bytes to string")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Error while parsing version")]
    VersionError(#[from] VersionError),

    #[error("IO error while trying to detect tool")]
    IOError(#[from] ioerror::IOError),
}

pub type Result<T> = core::result::Result<T, ToolDetectionError>;
