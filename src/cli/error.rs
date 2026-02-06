use thiserror::Error;

use crate::{installed_packages::InstalledPackagesError, installer::error::InstallerError, verifier::VerifierError};

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Error in installer: {0}")]
    InstallerError(#[from] InstallerError),

    #[error("Error while retrieving installed packages info: {0}")]
    InstalledPackagesError(#[from] InstalledPackagesError),

    #[error("Cannot read install directory: {0}")]
    VerifierError(#[from] VerifierError),
}
