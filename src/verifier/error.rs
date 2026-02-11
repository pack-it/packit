use thiserror::Error;

use crate::{cli::display::error::DisplayError, installer::error::InstallerError};

/// The errors that occur during verification.
#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Could not verify")]
    IOError(#[from] std::io::Error),

    #[error("Could not display issues")]
    DisplayError(#[from] DisplayError),

    #[error("Could not fix issues")]
    InstallerError(#[from] InstallerError),
}
