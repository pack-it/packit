use thiserror::Error;

use crate::{
    cli::display::error::DisplayError,
    installer::{builder::BuilderError, scripts::ScriptError},
    platforms::symlink::SymlinkError,
    repositories::error::RepositoryError,
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum InstallerError {
    //TODO: Split out errors
    #[error("Platform not found in targets.")]
    TargetError,

    #[error("Cannot write prefix directory due to incorrect permissions.")]
    PermissionsError,

    #[error("Error while interacting with filesystem\nCaused by: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Could not uninstall package '{package_name}'. {e}")]
    UninstallError {
        package_name: String,
        e: std::io::Error,
    },

    #[error("Installed package '{package_name} {}' does not exist.", version.clone().unwrap_or_default())]
    InstalledExistError {
        package_name: String,
        version: Option<String>,
    },

    #[error("Cannot delete package, '{package_name}' is a dependency.")]
    DependencyError {
        package_name: String,
    },

    #[error("Cannot delete external package '{package_name}'.")]
    ExternalError {
        package_name: String,
    },

    #[error("Cannot install package because the '{0}' dependency cannot be satisfied by the current package repository.")]
    SupportError(String),

    // Wrapped custom errors
    #[error("Cannot find a repository for installation\nCaused by: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot display installer error\nCaused by: {0}")]
    DisplayError(#[from] DisplayError),

    #[error("Cannot execute script\nCaused by: {0}")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot execute symlink opperation\nCaused by: {0}")]
    SymlinkError(#[from] SymlinkError),

    #[error("Cannot build package\nCaused by: {0}")]
    BuildError(#[from] BuilderError),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
