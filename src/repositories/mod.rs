use thiserror::Error;

pub mod types;
pub mod repository;
pub mod provider;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Cannot read repository file from disk")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot parse repository file: {message}", message = ._0.message())]
    ParseError(#[from] toml::de::Error),
}

pub(self) type Result<T> = std::result::Result<T, RepositoryError>;
