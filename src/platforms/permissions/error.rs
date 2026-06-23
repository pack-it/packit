// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{platforms::permissions::PlatformError, utils::ioerror};

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Group does not exist")]
    GroupDoesNotExist,

    #[error("Error during platform specific operations")]
    PlatformError(#[from] PlatformError),

    #[error("Error while fetching permissions")]
    IOError(#[from] ioerror::IOError),
}

pub type Result<T> = core::result::Result<T, PermissionError>;
