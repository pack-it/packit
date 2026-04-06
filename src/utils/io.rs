// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::Path};

use crate::{cli::display::logging::warning, platforms::symlink};

/// Recursively creates symlinks from a link directory to the original directory.
/// Note that there is an early return when the original doesn't exist. Non-existant link directories are created.
pub fn create_folder_symlinks(original_dir: &Path, link_dir: &Path) -> symlink::Result<()> {
    // Create destination if it does not exist
    if !link_dir.exists() {
        fs::create_dir_all(&link_dir)?;
    }

    // Skip symlinking if source does not exist
    if !original_dir.exists() {
        return Ok(());
    }

    // Symlink files
    for file in fs::read_dir(original_dir)? {
        let file = file?;

        let link_path = link_dir.join(file.file_name());

        // Create the symlinks for the subdirectory
        if file.file_type()?.is_dir() {
            create_folder_symlinks(&file.path(), &link_path)?;
            continue;
        }

        // Check if file already exists
        if fs::exists(&link_path)? {
            warning!("Symlink {:?} already exists in {:?}", file.file_name(), link_dir);
            continue;
        }

        // Symlink file in link path
        symlink::create_symlink(&file.path(), &link_path)?;
    }

    Ok(())
}

/// Searches for symlinks with a certain destination (destinations inside of the destination are also a match) and removes them.
pub fn remove_symlinks(search_dir: &Path, destination_dir: &Path) -> symlink::Result<()> {
    if !search_dir.exists() {
        return Ok(());
    }

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
