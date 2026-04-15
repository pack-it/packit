// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::platforms::permissions::PlatformError;

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Group does not exist")]
    GroupDoesNotExist,

    #[error("Error while fetching permissions")]
    IOError(#[from] std::io::Error),

    #[error("Error during platform specific operations")]
    PlatformError(#[from] PlatformError),
}

pub type Result<T> = core::result::Result<T, PermissionError>;
