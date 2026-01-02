use std::path::Path;

use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum SymlinkError {
    #[error("Symlink IO failed: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Path is not a symlink")]
    NonSymlink,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn create_symlink(source: &Path, destination: &Path) -> Result<(), SymlinkError> {
    std::os::unix::fs::symlink(source, destination)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn create_symlink(source: &Path, destination: &Path) -> Result<(), SymlinkError> {
    match source.is_dir() {
        true => std::os::windows::fs::symlink_dir(source, destination)?,
        false => std::os::windows::fs::symlink_file(source, destination)?,
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn create_symlink(source: &Path, destination: &Path) -> Result<(), SymlinkError> {
    panic!("Cannot create link for target, target is not supported.");
}

pub fn remove_symlink(symlink: &Path) -> Result<(), SymlinkError> {
    if !symlink.is_symlink() {
        return Err(SymlinkError::NonSymlink);
    }

    std::fs::remove_file(symlink)?;

    Ok(())
}
