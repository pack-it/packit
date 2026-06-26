// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

use crate::{
    installer::types::{PackageId, Version},
    utils::ioerror,
};

/// The errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot find repository '{repository_id}'")]
    RepositoryNotFoundError {
        repository_id: String,
    },

    #[error("Repository '{repository_id}' not supported")]
    RepositoryNotSupported {
        repository_id: String,
    },

    #[error("Cannot find package '{package_name}' with version '{}': {reason}", version.as_deref().unwrap_or("any"))]
    PackageNotFoundError {
        package_name: String,
        version: Option<String>,
        reason: PackageNotFoundReason,
    },

    #[error("Cannot find prebuild of package '{package_id}' revision {revision}")]
    PrebuildNotFound {
        package_id: PackageId,
        revision: u64,
    },

    #[error("Package metadata is not valid")]
    ValidationError(String),

    #[error("Cannot find target for package.")]
    TargetError,

    #[error("Dependency '{dependency}' cannot be satisfied: {reason}")]
    DependencyNotFoundError {
        dependency: String,
        reason: PackageNotFoundReason,
    },

    #[error("No supported version for the current target could be found for package '{0}'.")]
    SupportError(String),

    #[error("Request returned an unsuccessful status code '{0}'.")]
    UnsuccessfulRequest(reqwest::StatusCode),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot request repository file from external repository")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot parse repository file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot parse checksum from hex")]
    ChecksumParseError(#[from] hex::FromHexError),

    #[error("Cannot parse url")]
    UrlParseError(#[from] url::ParseError),
}

pub(super) type Result<T> = std::result::Result<T, RepositoryError>;

/// The reasons why a package cannot be found.
#[derive(Error, Clone, Debug)]
pub enum PackageNotFoundReason {
    #[error("cannot be found in any repository")]
    NotFound,

    #[error("not available for the current target")]
    UnsupportedTarget,

    #[error("not supported for the current Packit version, requires Packit {requires}")]
    NotSupported {
        requires: Version,
    },
}

impl PackageNotFoundReason {
    /// Gets the primary reason from an iterator over reasons
    /// If the iterator has no items, PackageNotFoundReason::NotFound is returned.
    pub fn get_primary_reason<'a>(reasons: impl Iterator<Item = &'a PackageNotFoundReason>) -> PackageNotFoundReason {
        let mut primary = PackageNotFoundReason::NotFound;

        for next in reasons {
            match &primary {
                // If the primary is UnsupportedTarget, only use new one if the next is not NotFound
                PackageNotFoundReason::UnsupportedTarget => {
                    if !matches!(next, PackageNotFoundReason::NotFound) {
                        primary = next.clone();
                    }
                },

                // If the primary is NotSupported, only use new one if next is NotSupported and the required version is lower
                PackageNotFoundReason::NotSupported { requires } => {
                    if let PackageNotFoundReason::NotSupported { requires: requires_next } = next
                        && requires_next < requires
                    {
                        primary = next.clone();
                    }
                },

                // If the current primary is not found, always use the new one
                PackageNotFoundReason::NotFound => primary = next.clone(),
            }
        }

        primary
    }
}
