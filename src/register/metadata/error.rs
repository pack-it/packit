// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use thiserror::Error;

use crate::{repositories::error::RepositoryError, utils::ioerror};

#[derive(Error, Debug)]
pub enum LocalMetadataError {
    #[error("Cannot find metadata file '{file_path}' in source repository")]
    RepositoryMetadataFileNotFound {
        file_path: String,
    },

    #[error("Cannot find local metadata file '{}'", file_path.display())]
    LocalMetadataFileNotFound {
        file_path: PathBuf,
    },

    #[error("Cannot fetch package metadata from repository")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot parse local metadata file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot serialize local metadata file")]
    SerializeError(#[from] toml::ser::Error),
}

pub type Result<T> = core::result::Result<T, LocalMetadataError>;
