// SPDX-License-Identifier: GPL-3.0-only
use std::{
    fs,
    path::{self, Path, PathBuf},
    str::Utf8Error,
};

#[cfg(unix)]
use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

use crate::{
    cli::display::logging::{debug, warning},
    platforms::symlink,
    utils::ioerror::{self, IOResultExt},
};

/// Recursively creates symlinks from a link directory to the original directory.
/// Note that there is an early return when the original doesn't exist. Non-existant link directories are created.
pub fn create_folder_symlinks(original_dir: &Path, link_dir: &Path, overwrite: bool) -> symlink::Result<()> {
    // Skip symlinking if source does not exist
    if !original_dir.exists() {
        return Ok(());
    }

    // Create destination if it does not exist
    if !link_dir.exists() {
        fs::create_dir_all(link_dir).err_with_path("create dirs", link_dir)?;
    }

    // Symlink files
    for file in fs::read_dir(original_dir).err_with_path("read", original_dir)? {
        let file = file.err_with_path("iterate", original_dir)?;

        let link_path = link_dir.join(file.file_name());

        // Create the symlinks for the subdirectory
        if file.file_type().err_with_path("get file type of", file.path())?.is_dir() {
            create_folder_symlinks(&file.path(), &link_path, overwrite)?;
            continue;
        }

        // Check if file already exists
        if symlink::exists(&link_path) {
            // If the symlink already exists as expected, skip removing and recreation
            if link_path.is_symlink() && fs::read_link(&link_path).err_with_path("read link", &link_path)? == file.path() {
                continue;
            }

            // Show warning and continue if overwrite is disabled
            if !overwrite {
                warning!(
                    "Symlink '{}' already exists in '{}'",
                    file.file_name().display(),
                    link_dir.display()
                );
                continue;
            }

            // Remove existing symlink when overwrite is enabled
            debug!("Overwriting symlink '{}'", link_path.display());
            symlink::remove_symlink(&link_path)?;
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

    for file in fs::read_dir(search_dir).err_with_path("read", search_dir)? {
        let file = file.err_with_path("iterate", search_dir)?;
        let file_type = file.file_type().err_with_path("get file type of", file.path())?;

        if file_type.is_dir() {
            remove_symlinks(&file.path(), destination_dir)?;

            // Remove the directory if it is empty after removing symlinks
            if fs::read_dir(file.path()).err_with_path("read", file.path())?.next().is_none() {
                fs::remove_dir(file.path()).err_with_path("remove", file.path())?;
            }
        }

        if file_type.is_symlink() && fs::read_link(file.path()).err_with_path("read link", file.path())?.starts_with(destination_dir) {
            symlink::remove_symlink(&file.path())?
        }
    }

    Ok(())
}

/// Parses a path from an array of bytes.
/// Can return a Utf8Error on Windows.
pub fn parse_path_from_bytes(bytes: &[u8]) -> Result<&Path, Utf8Error> {
    #[cfg(unix)]
    let string = OsStr::from_bytes(bytes);

    #[cfg(not(unix))]
    let string = str::from_utf8(bytes)?;

    Ok(Path::new(string))
}

/// Checks recursively if a directory is empty (contains nothing but empty directories).
/// Returns true if empty, false if not.
pub fn directory_is_empty(directory: &Path) -> Result<bool, ioerror::IOError> {
    for package in directory.read_dir().err_with_path("read", directory)? {
        let package = package.err_with_path("iterate", directory)?;

        if !package.path().is_dir() {
            return Ok(false);
        }

        if !directory_is_empty(&package.path())? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Normalizes a path by resolving current and parent dir operators.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match &component {
            std::path::Component::ParentDir => {
                if let Some(path::Component::Normal(_)) = components.last() {
                    components.pop();
                    continue;
                }

                components.push(component);
            },
            std::path::Component::CurDir => (),
            std::path::Component::Prefix(_) | std::path::Component::RootDir | std::path::Component::Normal(_) => {
                components.push(component);
            },
        }
    }

    let mut path = PathBuf::new();
    for component in components {
        path.push(component.as_os_str());
    }

    path
}
