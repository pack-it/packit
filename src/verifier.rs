use std::{fs, path::Path};
use thiserror::Error;

use crate::config::Config;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Cannot read install directory\nCaused by: {0}")]
    IOError(#[from] std::io::Error),
}

/// TODO: Verifies if a specific package works
pub fn verify_package() {}

/// Checks if a package exists with the specified version (by checking the directory itself, not from information storage).
/// If the version is none, it checks if there is at least one package version with the specified name.
/// TODO: Implement
pub fn package_exists(package_name: &String, version: &Option<String>) -> bool {
    true
}

/// Gets the installed packages, by actually checking the install directory.
pub fn get_packages(config: &Config) -> Result<Vec<String>, VerifierError> {
    let packages_dir = Path::new(&config.prefix_directory).join("packages");

    // Look inside the packit install directory for package installations
    let mut packages: Vec<String> = Vec::new();
    let directory = fs::read_dir(packages_dir)?;
    for package_entry in directory {
        let package_entry = package_entry?;
        let path = package_entry.path();
        if path.is_file() {
            continue;
        }

        // Look inside the package installation for multiple versions
        let package_name = package_entry.file_name();
        for version_entry in fs::read_dir(path)? {
            let version_entry = version_entry?;
            if version_entry.path().is_file() {
                continue;
            }

            packages.push(package_name.to_str().unwrap().to_string() + " " + version_entry.file_name().to_str().unwrap());
        }
    }

    Ok(packages)
}

// TODO: Make a method that syncs info.toml and actuall directory (in case of out of sync)
