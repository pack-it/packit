use thiserror::Error;

use crate::{cli::display::error::DisplayError, installer::error::InstallerError, packager::PackagerError};

/// The errors that occur during verification.
#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Could not verify")]
    IOError(#[from] std::io::Error),

    #[error("Could not display issues")]
    DisplayError(#[from] DisplayError),

    #[error("Could not fix issues, because of an issue in the installer")]
    InstallerError(#[from] InstallerError),

    #[error("Could not verify, because of an issue in the packager")]
    PackagerError(#[from] PackagerError),
}
