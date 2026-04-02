// SPDX-License-Identifier: GPL-3.0-only
use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

/// The errors that occur during symlink operations.
#[derive(Error, Debug)]
pub enum SymlinkError {
    #[error("Path is not a symlink")]
    NonSymlink,

    #[error("The given symlink original '{original}' cannot be found.")]
    OriginalNotFound {
        original: PathBuf,
    },

    #[error("Symlink IO failed")]
    IOError(#[from] std::io::Error),
}

/// Creates a symlink at `link`, pointing to `original`. Checks if the original exists and calls platform specific code.
pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
    if !fs::exists(original)? {
        return Err(SymlinkError::OriginalNotFound {
            original: original.to_path_buf(),
        });
    }

    platform::create_symlink(original, link)?;
    Ok(())
}

/// Removes the given symlink. This is platform independent.
pub fn remove_symlink(symlink: &Path) -> Result<(), SymlinkError> {
    if !symlink.is_symlink() {
        return Err(SymlinkError::NonSymlink);
    }

    std::fs::remove_file(symlink)?;

    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub mod platform {
    use std::path::Path;

    use super::SymlinkError;

    /// Creates a symlink on macOS and Linux from link to original.
    pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
        std::os::unix::fs::symlink(original, link)?;

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub mod platform {
    use std::path::Path;

    use super::SymlinkError;

    /// Creates a symlink on Windows from link to original.
    pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
        match original.is_dir() {
            true => std::os::windows::fs::symlink_dir(original, link)?,
            false => std::os::windows::fs::symlink_file(original, link)?,
        }

        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub mod platform {
    use std::path::Path;

    use super::SymlinkError;

    /// Panics for any unsupported OS.
    pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
        panic!("Cannot create link for target, target is not supported.");
    }
}
