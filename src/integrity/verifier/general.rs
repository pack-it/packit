// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, path::PathBuf, str::FromStr};

use crate::{
    config::Config,
    installer::types::{PackageId, PackageName, Version},
    integrity::{Issue, error::Result},
    platforms::permissions::does_packit_group_exist,
    register::package_register::PackageRegister,
    utils::{io::directory_is_empty, ioerror::IOResultExt},
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
    for directory in fs::read_dir(&package_directory).err_with_path("read", &package_directory)? {
        let directory = directory.err_with_path("iterate", &package_directory)?;
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

        for version_directory in fs::read_dir(directory.path()).err_with_path("read", directory.path())? {
            let version_directory = version_directory.err_with_path("iterate", directory.path())?;
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

/// Checks for invalid files (files which shouldn't exist in a certain directory).
/// Returns an `Issue::InvalidFiles` if invalid files were found, `None` otherwise.
pub fn check_invalid_files(packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
    let mut invalid = Vec::new();

    for package in packages {
        invalid.extend(check_invalid_dependencies_files(package, register, config)?);
    }

    // Check if the dependencies directory has wrong files
    let dependencies_dir = config.prefix_directory.join("dependencies");
    for file in fs::read_dir(&dependencies_dir).err_with_path("read", &dependencies_dir)? {
        let file = file.err_with_path("iterate", &dependencies_dir)?;

        // Check if the file is a directory
        if let Ok(file_type) = file.file_type() {
            if !file_type.is_dir() {
                invalid.push(file.path());
                continue;
            }
        }

        // Get the file name (if it's not a valid unicode file name it shouldn't exist here)
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            invalid.push(file.path());
            continue;
        };

        // Get the package id (if it's not a valid package id it shouldn't exist here)
        let Ok(package_id) = PackageId::from_str(file_name) else {
            invalid.push(file.path());
            continue;
        };

        // If the file doesn't correspond to any installed packages it shouldn't be here
        if register.iterate_all().all(|p| p.package_id != package_id) {
            invalid.push(file.path());
        }
    }

    if invalid.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::InvalidFiles(invalid)))
}

/// Checks for invalid files in the dependencies directory of a given package.
/// Returns a list of invalid files (which could be empty).
fn check_invalid_dependencies_files(package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<Vec<PathBuf>> {
    let mut invalid = Vec::new();

    // If the package cannot be found we can't check dir dependencies issues
    let Some(package_version) = register.get_package_version(package_id) else {
        return Ok(invalid);
    };

    // Don't read the directory if the package has no dependencies
    if package_version.dependencies.is_empty() {
        return Ok(invalid);
    }

    // Check for invalid files in the dependencies directory of the given package
    let dependencies_dir = config.prefix_directory.join("dependencies").join(package_id.to_string());
    for file in fs::read_dir(&dependencies_dir).err_with_path("read", &dependencies_dir)? {
        let file = file.err_with_path("iterate", &dependencies_dir)?;

        // Check if the file is a symlink
        if let Ok(file_type) = file.file_type() {
            if !file_type.is_symlink() {
                invalid.push(file.path());
                continue;
            }
        }

        // Get the file name (if it's not a valid unicode file name it shouldn't exist here)
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            invalid.push(file.path());
            continue;
        };

        // Get the package name (if it's not a valid package name it shouldn't exist here)
        let Ok(package_name) = PackageName::from_str(file_name) else {
            invalid.push(file.path());
            continue;
        };

        // If the file doesn't correspond to any dependencies it shouldn't be here
        if package_version.dependencies.iter().all(|d| d.name != package_name) {
            invalid.push(file.path());
        }
    }

    Ok(invalid)
}
