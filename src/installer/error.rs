use thiserror::Error;

use crate::{
    cli::display::error::DisplayError,
    installer::{builder::BuilderError, scripts::ScriptError, types::PackageIdError, unpack::UnpackError},
    platforms::symlink::SymlinkError,
    repositories::error::RepositoryError,
    storage::error::RegisterError,
    utils::tree::TreeError,
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Cannot write prefix directory due to incorrect permissions.")]
    PermissionsError,

    #[error("Prebuild checksum does not match")]
    ChecksumError,

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),

    #[error("Could not uninstall package '{package_name}'. {e}")]
    UninstallError {
        package_name: String,
        e: std::io::Error,
    },

    #[error("Package '{package_name}' with version '{}' does not exist.", version.clone().unwrap_or("any".to_string()))]
    PackageNotFound {
        package_name: String,
        version: Option<String>,
    },

    #[error("Cannot delete package, '{package_name}' is a dependency.")]
    DependencyError {
        package_name: String,
    },

    // Wrapped custom errors
    #[error("Cannot find a repository for installation")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot info package information")]
    RegisterError(#[from] RegisterError),

    #[error("Cannot display installer error")]
    DisplayError(#[from] DisplayError),

    #[error("Cannot execute script")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot execute symlink operation")]
    SymlinkError(#[from] SymlinkError),

    #[error("Cannot build package")]
    BuildError(#[from] BuilderError),

    #[error("Cannot unpack prebuild package")]
    UnpackError(#[from] UnpackError),

    #[error("Cannot (un)install because of invalid package id")]
    PackageIdError(#[from] PackageIdError),

    #[error("Cannot do tree operation")]
    TreeError(#[from] TreeError),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
