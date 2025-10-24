use thiserror::Error;

use crate::{installed_packages::InstalledPackagesError, repositories::error::RepositoryError};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Platform not found in targets.")]
    TargetError,

    #[error("Cannot request files for installation: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot find a repository for installation: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot store package information: {0}")]
    InstalledPackagesError(#[from] InstalledPackagesError),

    #[error("Cannot unpack response: {0}")]
    UnpackError(#[from] std::io::Error),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
