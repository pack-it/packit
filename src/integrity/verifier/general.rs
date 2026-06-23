// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, str::FromStr};

use crate::{
    config::Config,
    installer::types::{PackageName, Version},
    integrity::{Issue, error::Result},
    platforms::permissions::does_packit_group_exist,
    utils::io::directory_is_empty,
};

/// Checks if the packit group exists if multiuser mode is enabled in the config.
/// Returns the issue if the group does not exist, `None` otherwise.
pub fn check_packit_group(config: &Config) -> Result<Option<Issue>> {
    // We don't need the packit group if multiuser mode is not enabled
    if config.multiuser && !does_packit_group_exist()? {
        return Ok(Some(Issue::MissingPackitGroup));
    }

    Ok(None)
}

/// Checks for directories which shouldn't be in the `prefix/packages directory`.
/// This wil be any directory which is empty or doesn't have `<package-name>/<version>`.
/// Returns `None` if no stray directories are found, `Issue::StrayDirectories` otherwise.
pub fn check_stray_directories(config: &Config) -> Result<Option<Issue>> {
    let package_directory = config.prefix_directory.join("packages");
    let mut strays = HashSet::new();
    for directory in fs::read_dir(package_directory)? {
        let directory = directory?;
        if !directory.path().is_dir() {
            strays.insert(directory.path());
            continue;
        }

        // Try to get the package name
        let package_name = directory.file_name();
        let Some(package_name) = package_name.to_str() else {
            strays.insert(directory.path());
            continue;
        };

        // Try to create a `PackageName`
        if PackageName::from_str(package_name).is_err() {
            strays.insert(directory.path());
            continue;
        }

        // Check if the name directory is empty
        if directory_is_empty(&directory.path())? {
            strays.insert(directory.path());
            continue;
        }

        for version_directory in fs::read_dir(directory.path())? {
            let version_directory = version_directory?;
            if !version_directory.path().is_dir() {
                strays.insert(version_directory.path());
                continue;
            }

            // Try to get the version name
            let version_name = version_directory.file_name();
            let Some(version_str) = version_name.to_str() else {
                strays.insert(version_directory.path());
                continue;
            };

            // Try to create a `Version`
            if Version::from_str(version_str).is_err() {
                strays.insert(version_directory.path());
                continue;
            };

            // Check if the version directory is empty
            if directory_is_empty(&version_directory.path())? {
                strays.insert(version_directory.path());
                continue;
            }
        }
    }

    if strays.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::StrayDirectories(strays)))
}
