use thiserror::Error;

use crate::{cli::display::DisplayError, installed_packages::InstalledPackagesError, repositories::error::RepositoryError};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Platform not found in targets.")]
    TargetError,

    #[error("Cannot request files for installation: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot unpack response: {0}")]
    UnpackError(#[from] std::io::Error),

    #[error("Could not uninstall package '{package_name}'. {e}")]
    UninstallError {
        package_name: String,
        e: std::io::Error,
    },

    #[error("Installed package with name '{package_name}' and version '{version}' does not exist.")]
    InstalledExistError {
        package_name: String,
        version: String,
    },

    // Wrapped custom errors
    #[error("Cannot find a repository for installation: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot info package information: {0}")]
    InstalledPackagesError(#[from] InstalledPackagesError),

    #[error("Cannot display: {0}")]
    DisplayError(#[from] DisplayError),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
