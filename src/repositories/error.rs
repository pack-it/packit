use thiserror::Error;

/// The errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot read repository file from disk")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot request repository file from external repository")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot parse repository file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot find repository '{repository_id}'")]
    RepositoryNotFoundError {
        repository_id: String,
    },

    #[error("Cannot find package '{package_name}' with version '{}' in any repository", version.clone().unwrap_or("any".into()))]
    PackageNotFoundError {
        package_name: String,
        version: Option<String>,
    },

    #[error("Package is not valid")]
    ValidationError(String),

    #[error("Cannot find target for package.")]
    TargetError,

    #[error("The given package name is empty")]
    EmptyPackageName,

    #[error("Cannot parse checksum from bytes")]
    ChecksumParseError,
}

pub(super) type Result<T> = std::result::Result<T, RepositoryError>;
