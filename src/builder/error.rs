// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{
    builder::BinaryPatcherError,
    cli::display::styled::Styled,
    installer::{scripts::ScriptError, types::PackageName, unpack::UnpackError},
    platforms::tool_detection::error::ToolDetectionError,
    repositories::{error::RepositoryError, types::Requirement},
    utils::{ioerror, patches::PatchError},
};

/// The errors that occur during building.
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Dependency {} of type '{dependency_type}' is not installed.", package_name.style())]
    MissingDependencyError {
        dependency_type: String,
        package_name: PackageName,
    },

    #[error("Requirement '{requirement}' is not satisfied.\n{}", requirement.get_not_satisfied_message())]
    MissingRequirementError {
        requirement: Requirement,
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

    #[error("Error while detecting tool on the system")]
    ToolDetectionError(#[from] ToolDetectionError),

    #[error("Cannot request files for building")]
    RequestError(#[from] reqwest::Error),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot parse url of source")]
    UrlParseError(#[from] url::ParseError),
}

pub type Result<T> = core::result::Result<T, BuilderError>;
