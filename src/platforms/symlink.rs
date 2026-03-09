use std::path::{Path, PathBuf};

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

/// Creates a symlink on macOS and Linux from link to original.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
    std::os::unix::fs::symlink(original, link)?;

    Ok(())
}

/// Creates a symlink on Windows from link to original.
#[cfg(target_os = "windows")]
pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
    match original.is_dir() {
        true => std::os::windows::fs::symlink_dir(original, link)?,
        false => std::os::windows::fs::symlink_file(original, link)?,
    }

    Ok(())
}

/// Panics for any unsupported OS.
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn create_symlink(original: &Path, link: &Path) -> Result<(), SymlinkError> {
    panic!("Cannot create link for target, target is not supported.");
}

/// Removes the given symlink. This is platform independent.
pub fn remove_symlink(symlink: &Path) -> Result<(), SymlinkError> {
    if !symlink.is_symlink() {
        return Err(SymlinkError::NonSymlink);
    }

    std::fs::remove_file(symlink)?;

    Ok(())
}
