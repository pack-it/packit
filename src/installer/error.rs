// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{
    cli::display::error::DisplayError,
    installer::{
        builder::BuilderError,
        scripts::ScriptError,
        types::{PackageId, PackageIdError, Version},
        unpack::UnpackError,
    },
    platforms::{permissions::error::PermissionError, symlink::SymlinkError},
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

    #[error("Package '{package_name}' with version '{}' is not installed.", version.clone().unwrap_or("any".to_string()))]
    PackageNotFound {
        package_name: String,
        version: Option<String>,
    },

    #[error("Package '{package_id}' is already installed.")]
    AlreadyInstalledError {
        package_id: PackageId,
    },

    #[error("Multiple versions are installed, so only package name input is not specific enough. Please provide a version as well.")]
    SpecificityError,

    #[error("The new version '{new_version}' of this package cannot satisfy all dependents from the old package version.")]
    SatisfyError {
        new_version: Version,
    },

    #[error("Could not update, the given version '{new_version}' is lower then the current version.")]
    VersionTooLowError {
        new_version: Version,
    },

    #[error("Cannot delete package, '{package_name}' is a dependency.")]
    DependencyError {
        package_name: String,
    },

    #[error("Canceled package installation: {reason}.")]
    InstallationCanceled {
        reason: String,
    },

    #[error("An unreachable state has been reached: {msg}.")]
    UnreachableError {
        msg: String,
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

    #[error("Cannot set or get permissions")]
    PermissionError(#[from] PermissionError),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
