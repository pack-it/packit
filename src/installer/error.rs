use thiserror::Error;

use crate::{
    cli::display::error::DisplayError,
    installer::{builder::BuilderError, scripts::ScriptError},
    platforms::symlink::SymlinkError,
    repositories::error::RepositoryError,
    storage::error::InstalledPackagesError,
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Cannot write prefix directory due to incorrect permissions.")]
    PermissionsError,

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),

    #[error("Could not uninstall package '{package_name}'. {e}")]
    UninstallError {
        package_name: String,
        e: std::io::Error,
    },

    #[error("Package '{package_name}' with version '{}' does not exist.", version.clone().unwrap_or("any".to_string()))]
    InstalledExistError {
        package_name: String,
        version: Option<String>,
    },

    #[error("Cannot delete package, '{package_name}' is a dependency.")]
    DependencyError {
        package_name: String,
    },

    #[error("Cannot install package because the '{0}' dependency cannot be satisfied by the current package repository.")]
    SupportError(String),

    // Wrapped custom errors
    #[error("Cannot find a repository for installation")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot info package information")]
    InstalledPackagesError(#[from] InstalledPackagesError),

    #[error("Cannot display installer error")]
    DisplayError(#[from] DisplayError),

    #[error("Cannot execute script")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot execute symlink operation")]
    SymlinkError(#[from] SymlinkError),

    #[error("Cannot build package")]
    BuildError(#[from] BuilderError),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
