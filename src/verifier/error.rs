// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{
    cli::display::error::DisplayError,
    installer::{
        error::InstallerError,
        types::{PackageNameError, VersionError},
    },
    packager::PackagerError,
    platforms::symlink::SymlinkError,
};

/// The errors that occur during verification.
#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Cannot parse symlink name, because it doesn't end properly")]
    InvalidSymlink,

    #[error("Cannot parse filename, because it contains invalid unicode")]
    InvalidUnicodeError,

    #[error("Could not verify")]
    IOError(#[from] std::io::Error),

    #[error("Could not display issues")]
    DisplayError(#[from] DisplayError),

    #[error("Could not fix issues, because of an issue in the installer")]
    InstallerError(#[from] InstallerError),

    #[error("Could not verify, because of an issue in the packager")]
    PackagerError(#[from] PackagerError),

    #[error("Could not verify, because the version number couldn't be parsed")]
    VersionError(#[from] VersionError),

    #[error("Could not verify, because a symlink operation failed")]
    SymlinkError(#[from] SymlinkError),

    #[error("Could not verify because of an invalid package name")]
    PackageNameError(#[from] PackageNameError),
}

pub(super) type Result<T> = std::result::Result<T, VerifierError>;
