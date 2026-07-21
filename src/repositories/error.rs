// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use thiserror::Error;

use crate::{
    cli::display::styled::Styled,
    installer::types::{PackageId, PackageName, Version},
    repositories::types::Date,
    utils::ioerror,
};

/// The errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot find repository '{repository_id}'")]
    RepositoryNotFoundError {
        repository_id: String,
    },

    #[error("Repository '{repository_id}' not supported, the current Packit version is too low")]
    RepositoryNotSupported {
        repository_id: String,
    },

    #[error("Cannot find package {} with version '{}': {reason}", package_name.style(), version.as_ref().map_or("any".normal(), |v| v.style()))]
    PackageNotFoundError {
        package_name: PackageName,
        version: Option<Version>,
        reason: PackageNotFoundReason,
    },

    #[error("Cannot find prebuild '{prebuild_id}' of package {} revision {revision}", package_id.style())]
    PrebuildNotFound {
        prebuild_id: String,
        package_id: PackageId,
        revision: u64,
    },

    #[error("Package metadata is not valid: {0}")]
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

    #[error("package is disabled since {since}{}", reason.as_ref().map(|reason| format!(", with reason '{reason}'")).unwrap_or_default())]
    Disabled {
        since: Date,
        reason: Option<String>,
    },

    #[error("not available for the current target")]
    UnsupportedTarget,

    #[error("not supported for the current Packit version, requires Packit {requires}")]
    NotSupported {
        requires: Version,
    },
}

impl PackageNotFoundReason {
    /// Gets the primary reason from an iterator over reasons
    /// If the iterator has no items, `PackageNotFoundReason::NotFound` is returned.
    pub fn get_primary_reason<'a>(reasons: impl Iterator<Item = &'a PackageNotFoundReason>) -> PackageNotFoundReason {
        let mut primary = PackageNotFoundReason::NotFound;

        for next in reasons {
            match &primary {
                // If the primary is `UnsupportedTarget`, only use the new one if the next is not `NotFound` or `Disabled`
                PackageNotFoundReason::UnsupportedTarget => {
                    if !matches!(next, PackageNotFoundReason::NotFound | PackageNotFoundReason::Disabled { .. }) {
                        primary = next.clone();
                    }
                },

                // If the primary is `NotSupported`, only use the new one if next is `NotSupported` and the required version is lower
                PackageNotFoundReason::NotSupported { requires } => {
                    if let PackageNotFoundReason::NotSupported { requires: requires_next } = next
                        && requires_next < requires
                    {
                        primary = next.clone();
                    }
                },

                // If the primary is `Disabled`, only use the new one if the next if `UnsupportedTarget` or `NotSupported`, or a newer disable
                PackageNotFoundReason::Disabled { since, .. } => {
                    match next {
                        PackageNotFoundReason::UnsupportedTarget | PackageNotFoundReason::NotSupported { .. } => primary = next.clone(),
                        PackageNotFoundReason::Disabled { since: since_next, .. } if since_next > since => primary = next.clone(),
                        _ => {},
                    };
                },

                // If the current primary is `NotFound`, always use the new one
                PackageNotFoundReason::NotFound => primary = next.clone(),
            }
        }

        primary
    }
}
