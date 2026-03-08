use thiserror::Error;

use crate::installer::types::PackageId;

/// The errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot find repository '{repository_id}'")]
    RepositoryNotFoundError {
        repository_id: String,
    },

    #[error("Cannot find package '{package_name}' with version '{}' in any repository", version.clone().unwrap_or("any".into()))]
    PackageNotFoundError {
        package_name: String,
        version: Option<String>,
    },

    #[error("Cannot find prebuild of package '{package_id}' revision {revision}")]
    PrebuildNotFound {
        package_id: PackageId,
        revision: u64,
    },

    #[error("Package is not valid")]
    ValidationError(String),

    #[error("Cannot find target for package.")]
    TargetError,

    #[error("The given package name is empty")]
    EmptyPackageName,

    #[error("Dependency '{0}' cannot be satisfied by the current package repository for the current target.")]
    DependencySupportError(String),

    #[error("No supported version for the current target could be found for package '{0}'.")]
    SupportError(String),

    #[error("Cannot read repository file from disk")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot request repository file from external repository")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot parse repository file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot parse checksum from hex")]
    ChecksumParseError(#[from] hex::FromHexError),
}

pub(super) type Result<T> = std::result::Result<T, RepositoryError>;
