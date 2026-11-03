// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf};

// TODO: Add print statements for failure, for easy trace

/// Checks the package version existence in the `packages` directory.
/// Returns true if the package and versions exist.
pub fn check_package_directory(prefix: &PathBuf, package_name: &str, versions: &[&str]) -> bool {
    let local_metadata_directory = prefix.join("packages");
    check_package_version_existence(&local_metadata_directory, package_name, versions)
}

/// Checks the package version existence in the `metadata` directory.
/// Returns true if the package and versions exist.
pub fn check_local_metadata(prefix: &PathBuf, package_name: &str, versions: &[&str]) -> bool {
    let local_metadata_directory = prefix.join("metadata");
    check_package_version_existence(&local_metadata_directory, package_name, versions)
}

/// Checks if the package directory exists and that only the given versions exist.
/// Returns true if the package and versions exist. False otherwise.
fn check_package_version_existence(directory: &PathBuf, package_name: &str, versions: &[&str]) -> bool {
    let package_directory = directory.join(package_name);
    if !package_directory.exists() {
        return false;
    }

    // Check if all directories are in the given versions
    for entry in fs::read_dir(&package_directory).expect("Expected successful directory read") {
        let entry = entry.expect("Expected valid entry");
        let metadata = entry.metadata().expect("Expected valid metadata");

        // Check if the entry is a directory
        if !metadata.is_dir() {
            return false;
        }

        let file_name = entry.file_name().to_ascii_lowercase();
        let Some(file_name) = file_name.to_str() else { return false };

        if !versions.contains(&file_name) {
            return false;
        }
    }

    true
}

/// Returns true if the given package version is actually the active version, false otherwise.
pub fn check_active_package(prefix: &PathBuf, package_name: &str, version: &str) -> bool {
    let active_symlink = prefix.join("active").join(package_name);
    let expected_destination = prefix.join("packages").join(package_name).join(version);

    if !active_symlink.exists() {
        return false;
    }

    let Ok(destination) = fs::read_link(active_symlink) else {
        // TODO: print symlink not existing or not a symlink
        return false;
    };

    destination == expected_destination
}
