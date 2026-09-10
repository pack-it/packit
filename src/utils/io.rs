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
/// Note that there is an early return when the original doesn't exist. Non-existent link directories are created.
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

        // Check if symlink already exists
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
            symlink::remove_symlink(&file.path())?;
        }
    }

    Ok(())
}

/// Parses a path from an array of bytes.
/// Can return a `Utf8Error` on `Windows`.
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

/// Gets the last extension of the given file name, or `None` if the file has no extension.
pub fn get_last_extension(file_name: &str) -> Option<&str> {
    let extension_index = file_name.chars().rev().position(|x| x == '.')?;
    let (_, extension) = file_name.split_at(file_name.len() - extension_index - 1);

    Some(extension)
}

#[cfg(test)]
pub mod tests {

    use std::{fs::File, str::FromStr};

    use tempfile::tempdir;

    use crate::platforms::symlink::create_symlink;

    use super::*;

    #[test]
    fn create_folder_symlinks_test() {
        let original = tempdir().unwrap();
        let original = original.path();
        File::create(original.join("test.txt")).unwrap();
        let foo_dir = original.join("foo");
        fs::create_dir(&foo_dir).unwrap();
        File::create(foo_dir.join("bar.txt")).unwrap();

        let destination = tempdir().unwrap();
        assert!(create_folder_symlinks(original, destination.path(), false).is_ok());
        assert!(symlink::exists(&destination.path().join("test.txt")));
        assert!(symlink::exists(&destination.path().join("foo").join("bar.txt")));
    }

    #[test]
    fn create_folder_symlinks_missing_destination() {
        let original = tempdir().unwrap();
        File::create(original.path().join("test.txt")).unwrap();
        let missing_destination = tempdir().unwrap().path().join("missing").join("more");

        assert!(create_folder_symlinks(original.path(), &missing_destination, false).is_ok());
        assert!(symlink::exists(&missing_destination.join("test.txt")));
    }

    #[test]
    fn create_folder_symlinks_missing_original() {
        let original = tempdir().unwrap();
        let original = original.path().join("does").join("not").join("exist");
        let link = tempdir().unwrap();
        assert!(create_folder_symlinks(&original, &link.path().join("foo"), false).is_ok());
        assert!(!original.exists());
    }

    #[test]
    fn create_equal_existing_folder_symlinks() {
        let original = tempdir().unwrap();
        File::create(original.path().join("test.txt")).unwrap();
        let destination = tempdir().unwrap();

        assert!(create_folder_symlinks(original.path(), destination.path(), false).is_ok());
        File::create(original.path().join("test2.txt")).unwrap();
        assert!(create_folder_symlinks(original.path(), destination.path(), false).is_ok());
        assert!(symlink::exists(&destination.path().join("test.txt")));
        assert!(symlink::exists(&destination.path().join("test2.txt")));
    }

    #[test]
    fn create_existing_folder_symlinks() {
        let original = tempdir().unwrap();
        File::create(original.path().join("test.txt")).unwrap();
        File::create(original.path().join("other.txt")).unwrap();
        let destination = tempdir().unwrap();
        let destination_test_path = destination.path().join("test.txt");
        create_symlink(&original.path().join("other.txt"), &destination_test_path).unwrap();

        assert!(create_folder_symlinks(original.path(), destination.path(), false).is_ok());
        let resulting_link = fs::read_link(&destination_test_path).err_with_path("read link", &destination_test_path).unwrap();
        assert_eq!(resulting_link, original.path().join("other.txt"));
        assert!(symlink::exists(&destination.path().join("other.txt")));
    }

    #[test]
    fn force_create_existing_folder_symlinks() {
        let original = tempdir().unwrap();
        File::create(original.path().join("test.txt")).unwrap();
        File::create(original.path().join("other.txt")).unwrap();
        let destination = tempdir().unwrap();
        let destination_test_path = destination.path().join("test.txt");
        create_symlink(&original.path().join("other.txt"), &destination_test_path).unwrap();

        assert!(create_folder_symlinks(original.path(), destination.path(), true).is_ok());
        let resulting_link = fs::read_link(&destination_test_path).err_with_path("read link", &destination_test_path).unwrap();
        assert_eq!(resulting_link, original.path().join("test.txt"));
        assert!(symlink::exists(&destination.path().join("other.txt")));
    }

    #[test]
    fn remove_symlinks_test() {
        let destination_dir = tempdir().unwrap();
        let test_file = destination_dir.path().join("test.txt");
        File::create(&test_file).unwrap();
        fs::create_dir(&destination_dir.path().join("foo")).unwrap();
        let test2_file = destination_dir.path().join("test2.txt");
        File::create(&test2_file).unwrap();

        let temp_dir = tempdir().unwrap();
        let temp_foo_dir = temp_dir.path().join("foo");
        fs::create_dir(&temp_foo_dir).unwrap();
        create_symlink(&test_file, &temp_dir.path().join("test.txt")).unwrap();
        create_symlink(&test2_file, &temp_foo_dir.join("test.txt")).unwrap();

        assert!(remove_symlinks(temp_dir.path(), destination_dir.path()).is_ok());
        assert!(!symlink::exists(&temp_dir.path().join("test.txt")));
        assert!(!symlink::exists(&temp_foo_dir.join("test.txt")));
        assert!(!temp_foo_dir.exists());
    }

    #[test]
    fn remove_symlinks_missing_dirs() {
        let temp_dir = tempdir().unwrap();
        let other_dir = tempdir().unwrap();
        assert!(remove_symlinks(&temp_dir.path().join("/missing"), &other_dir.path()).is_ok());
        assert!(remove_symlinks(&other_dir.path(), &temp_dir.path().join("/missing")).is_ok());
    }

    #[test]
    fn directory_is_empty_test() {
        let temp_dir = tempdir().unwrap();
        let temp_dir = temp_dir.path();
        assert!(directory_is_empty(temp_dir).unwrap());

        let foo_dir = temp_dir.join("foo");
        fs::create_dir(&foo_dir).unwrap();
        assert!(directory_is_empty(&temp_dir).unwrap());

        let bar_dir = temp_dir.join("bar");
        fs::create_dir(&bar_dir).unwrap();
        assert!(directory_is_empty(&temp_dir).unwrap());

        File::create(bar_dir.join("test.txt")).unwrap();
        assert!(!directory_is_empty(&temp_dir).unwrap());
    }

    #[test]
    fn directory_is_empty_with_symlink() {
        let temp_dir = tempdir().unwrap();
        let temp_dir = temp_dir.path();
        let file_path = temp_dir.join("test.txt");
        File::create(&file_path).unwrap();

        let other_temp_dir = tempdir().unwrap();
        let other_temp_dir = other_temp_dir.path();
        assert!(directory_is_empty(other_temp_dir).unwrap());
        create_symlink(&file_path, &other_temp_dir.join("other_test.txt")).unwrap();
        assert!(!directory_is_empty(other_temp_dir).unwrap());
    }

    #[test]
    fn normalize_path_test() {
        assert_eq!(
            normalize_path(&PathBuf::from_str("foo/baz/../bar").unwrap()),
            PathBuf::from_str("foo/bar").unwrap()
        );

        assert_eq!(
            normalize_path(&PathBuf::from_str("./foo/./bar").unwrap()),
            PathBuf::from_str("foo/bar").unwrap()
        );

        assert_eq!(
            normalize_path(&PathBuf::from_str("../foo").unwrap()),
            PathBuf::from_str("../foo").unwrap()
        );

        assert_eq!(
            normalize_path(&PathBuf::from_str("foo/../../bar").unwrap()),
            PathBuf::from_str("../bar").unwrap()
        );

        assert_eq!(
            normalize_path(&PathBuf::from_str("foo/../../../..").unwrap()),
            PathBuf::from_str("../../..").unwrap()
        );
    }

    #[test]
    fn empty_normalize() {
        assert_eq!(
            normalize_path(&PathBuf::from_str("foo/..").unwrap()),
            PathBuf::from_str("").unwrap()
        );

        assert_eq!(normalize_path(&PathBuf::from_str(".").unwrap()), PathBuf::from_str("").unwrap());
    }

    #[test]
    fn get_last_extension_test() {
        assert_eq!(get_last_extension("test.x"), Some(".x"));
        assert_eq!(get_last_extension("test.x.y"), Some(".y"));
        assert_eq!(get_last_extension(".y"), Some(".y"));
        assert_eq!(get_last_extension("."), Some("."));
        assert_eq!(get_last_extension("test."), Some("."));
        assert_eq!(get_last_extension("test"), None);
        assert_eq!(get_last_extension(""), None);
    }
}
