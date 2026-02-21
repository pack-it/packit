use std::{fs, path::Path};

use thiserror::Error;

use crate::{cli::display::logging::warning, platforms::symlink};

/// The errors that occur during symlink opperations.
#[derive(Error, Debug)]
pub enum SymlinkError {
    #[error("Symlink IO failed")]
    IOError(#[from] std::io::Error),

    #[error("Path is not a symlink")]
    NonSymlink,
}

pub fn create_folder_symlinks(source_dir: &Path, destination_dir: &Path, keep_subdirectories: bool) -> Result<(), SymlinkError> {
    // Create destination if it does not exist
    if !destination_dir.exists() {
        fs::create_dir_all(&destination_dir)?;
    }

    // Skip symlinking if source does not exist
    if !source_dir.exists() {
        return Ok(());
    }

    // Symlink files
    for file in fs::read_dir(source_dir)? {
        let file = file?;

        let destination = destination_dir.join(file.file_name());

        // Handle directories
        if file.file_type()?.is_dir() {
            // If we want to keep subdirectories, create the symlinks for the subdirectory
            // TODO: Handle subdirectories properly
            if keep_subdirectories {
                create_folder_symlinks(&file.path(), &destination, true)?;
            } else {
                dbg!("Skipping subdirectory", file);
            }

            continue;
        }

        // Check if file already exists
        if fs::exists(&destination)? {
            warning!("Symlink {:?} already exists in {:?}", file.file_name(), destination_dir);
            continue;
        }

        // Symlink file in destination directory
        symlink::create_symlink(&file.path(), &destination)?;
    }

    Ok(())
}

/// Searches for symlinks with a certain destination (destinations inside of the destination are also a match).
pub fn remove_symlinks(search_dir: &Path, destination_dir: &Path) -> Result<(), SymlinkError> {
    for file in fs::read_dir(search_dir)? {
        let file = file?;
        let file_type = file.file_type()?;

        if file_type.is_dir() {
            remove_symlinks(&file.path(), destination_dir)?;

            // Remove the directory if it is empty after removing symlinks
            if fs::read_dir(file.path())?.next().is_none() {
                fs::remove_dir(file.path())?;
            }
        }

        if file_type.is_symlink() && fs::read_link(file.path())?.starts_with(destination_dir) {
            symlink::remove_symlink(&file.path())?
        }
    }

    Ok(())
}
