// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::utils::ioerror;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Failed to read user input")]
    IOError(#[source] ioerror::IOError),
}

pub type Result<T> = core::result::Result<T, DisplayError>;
