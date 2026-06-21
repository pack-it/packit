// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{
    builder::BinaryPatcherError,
    installer::{scripts::ScriptError, unpack::UnpackError},
    repositories::error::RepositoryError,
    utils::{ioerror, patches::PatchError},
};

/// The errors that occur during building.
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Dependency '{package_name}' of type '{dependency_type}' is not installed.")]
    MissingDependencyError {
        dependency_type: String,
        package_name: String,
    },

    #[error("Checksum does not match")]
    ChecksumError,

    #[error("The source url has an empty path")]
    EmptyUrlPath,

    #[error("The required patch was not found in the repository")]
    RepositoryPatchNotFound,

    #[error("Build files download unsuccessful, with status code: {0}.")]
    RequestUnsuccessful(reqwest::StatusCode),

    #[error("Cannot unpack response")]
    UnpackError(#[from] UnpackError),

    #[error("Cannot execute build script")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot find a repository for building")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot patch binaries")]
    PatchBinaryError(#[from] BinaryPatcherError),

    #[error("Cannot apply patch file")]
    ApplyPatchError(#[from] PatchError),

    #[error("Cannot request files for building")]
    RequestError(#[from] reqwest::Error),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot parse url of source")]
    UrlParseError(#[from] url::ParseError),
}

pub type Result<T> = core::result::Result<T, BuilderError>;
