use thiserror::Error;

/// The errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot read repository file from disk: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot request repository file from external repository: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot parse repository file: {0}")]
    ParseError(#[from] toml::de::Error),
}

pub(super) type Result<T> = std::result::Result<T, RepositoryError>;
