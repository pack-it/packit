// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use thiserror::Error;

/// Small std::io::Error wrapper with better messages
#[derive(Error, Debug)]
pub enum IOError {
    #[error(transparent)]
    Standard(std::io::Error),

    #[error("Failed to {operation}")]
    Operation {
        operation: String,

        #[source]
        source: std::io::Error,
    },

    #[error("Failed to {operation} '{}'", path.display())]
    WithPath {
        operation: String,
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },
}

pub trait IOResultExt<T> {
    /// Wraps the std::io::Error directly, without context.
    fn err_std(self) -> Result<T, IOError>;

    /// Wraps the std::io::Error directly, with an operation.
    fn err_operation(self, operation: &str) -> Result<T, IOError>;

    /// Wraps the std::io::Error directly, with an operation and path context.
    fn err_with_path(self, operation: &str, path: impl Into<PathBuf>) -> Result<T, IOError>;
}

impl<T> IOResultExt<T> for std::io::Result<T> {
    fn err_std(self) -> Result<T, IOError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(IOError::Standard(e)),
        }
    }

    fn err_operation(self, operation: &str) -> Result<T, IOError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(IOError::Operation {
                operation: operation.into(),
                source: e,
            }),
        }
    }

    fn err_with_path(self, operation: &str, path: impl Into<PathBuf>) -> Result<T, IOError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(IOError::WithPath {
                operation: operation.into(),
                path: path.into(),
                source: e,
            }),
        }
    }
}
