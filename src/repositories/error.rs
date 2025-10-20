use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot read repository file from disk: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot parse repository file: {message}", message = ._0.message())]
    ParseError(#[from] toml::de::Error),
}

pub(super) type Result<T> = std::result::Result<T, RepositoryError>;
