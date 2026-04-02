// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Could not display")]
    IOError(#[from] std::io::Error),
}
