// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use thiserror::Error;

use crate::{
    builder::error::BuilderError,
    cli::display::{error::DisplayError, styled::Styled},
    installer::{
        scripts::ScriptError,
        types::{PackageId, PackageIdError, PackageName, Version},
        unpack::UnpackError,
    },
    platforms::{permissions::error::PermissionError, symlink::SymlinkError},
    register::error::RegisterError,
    repositories::error::RepositoryError,
    utils::{ioerror, tree::TreeError},
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Cannot write prefix directory due to incorrect permissions.")]
    PermissionsError,

    #[error("Prebuild checksum does not match")]
    ChecksumError,

    #[error("Package {} with version '{}' is not installed.", package_name.style(), version.as_ref().map_or("any".normal(), |v| v.style()))]
    PackageNotFound {
        package_name: PackageName,
        version: Option<Version>,
    },

    #[error("Package {} is already installed.", package_id.style())]
    AlreadyInstalledError {
        package_id: PackageId,
    },

    #[error("The new version {} of this package cannot satisfy all dependents from the old package version.", new_version.style())]
    SatisfyError {
        new_version: Version,
    },

    #[error("Could not update, the given version {} is lower then the current version.", new_version.style())]
    VersionTooLowError {
        new_version: Version,
    },

    #[error("Cannot delete package, {} is a dependency.", package_name.style())]
    DependencyError {
        package_name: PackageName,
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
    #[error("Cannot fetch package metadata from repository")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot get information from register")]
    RegisterError(#[from] RegisterError),

    #[error("Error while displaying message on console")]
    DisplayError(#[from] DisplayError),

    #[error("Error while executing script")]
    ScriptError(#[from] ScriptError),

    #[error("Error while doing symlink operation")]
    SymlinkError(#[from] SymlinkError),

    #[error("Cannot build package")]
    BuildError(#[from] BuilderError),

    #[error("Cannot unpack prebuild package")]
    UnpackError(#[from] UnpackError),

    #[error("Cannot (un)install because of invalid package id")]
    PackageIdError(#[from] PackageIdError),

    #[error("Cannot do tree operation")]
    TreeError(#[from] TreeError),

    #[error("Error while setting or getting permissions")]
    PermissionError(#[from] PermissionError),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),
}

pub(super) type Result<T> = std::result::Result<T, InstallerError>;
