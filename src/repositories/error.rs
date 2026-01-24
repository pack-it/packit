use thiserror::Error;

/// The errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot read repository file from disk\nCaused by: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot request repository file from external repository\nCaused by: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot parse repository file\nCaused by: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot find repository '{repository_id}'")]
    RepositoryNotFoundError {
        repository_id: String,
    },

    #[error("Cannot find package '{package_name} {}' in any repository", version.clone().unwrap_or_default())]
    PackageNotFoundError {
        package_name: String,
        version: Option<String>,
    },

    #[error("Package is not valid\nCaused by: {0}")]
    ValidationError(String),
}

pub(super) type Result<T> = std::result::Result<T, RepositoryError>;
