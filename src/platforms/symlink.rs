// SPDX-License-Identifier: GPL-3.0-only
use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::utils::ioerror::{self, IOResultExt};

/// The errors that occur during symlink operations.
#[derive(Error, Debug)]
pub enum SymlinkError {
    #[error("Path is not a symlink")]
    NonSymlink,

    #[error("The given symlink original '{original}' cannot be found.")]
    OriginalNotFound {
        original: PathBuf,
    },

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),
}

pub type Result<T> = core::result::Result<T, SymlinkError>;

/// Creates a symlink at `link`, pointing to `original`. Checks if the original exists and calls platform specific code.
pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
    if !fs::exists(original).err_with_path("check existance of", original)? {
        return Err(SymlinkError::OriginalNotFound {
            original: original.to_path_buf(),
        });
    }

    platform::create_symlink(original, link)?;
    Ok(())
}

/// Removes the given symlink. Checks if the path is a symlink and calls platform specific code.
pub fn remove_symlink(symlink: &Path) -> Result<()> {
    if !symlink.is_symlink() {
        return Err(SymlinkError::NonSymlink);
    }

    platform::remove_symlink(symlink)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub mod platform {
    use std::path::Path;

    use crate::utils::ioerror::IOResultExt;

    use super::Result;

    /// Creates a symlink on macOS and Linux from link to original.
    pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
        std::os::unix::fs::symlink(original, link).err_with_path("create symlink", link)?;

        Ok(())
    }

    /// Removes the given symlink on macOS and Linux.
    pub fn remove_symlink(symlink: &Path) -> Result<()> {
        std::fs::remove_file(symlink).err_with_path("remove symlink", symlink)?;

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub mod platform {
    use std::path::Path;

    use crate::utils::ioerror::IOResultExt;

    use super::Result;

    /// Creates a symlink on Windows from link to original.
    pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
        match original.is_dir() {
            true => std::os::windows::fs::symlink_dir(original, link).err_with_path("create dir symlink", link)?,
            false => std::os::windows::fs::symlink_file(original, link).err_with_path("create file symlink", link)?,
        }

        Ok(())
    }

    /// Removes the given symlink on macOS and Linux.
    pub fn remove_symlink(symlink: &Path) -> Result<()> {
        match symlink.is_dir() {
            true => std::fs::remove_dir(symlink).err_with_path("remove dir symlink", symlink)?,
            false => std::fs::remove_file(symlink).err_with_path("remove file symlink", symlink)?,
        }

        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub mod platform {
    use std::path::Path;

    use super::Result;

    /// Panics for any unsupported OS.
    pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
        panic!("Cannot create link for target, target is not supported.");
    }

    /// Panics for any unsupported OS.
    pub fn remove_symlink(symlink: &Path) -> Result<()> {
        panic!("Cannot remove link for target, target is not supported.");
    }
}
