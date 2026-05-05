use std::{collections::HashSet, fs, str::FromStr};

use crate::{
    config::Config,
    installer::types::{PackageId, PackageName, Version},
    verifier::error::{Result, VerifierError},
};

/// Gets all the packages stored in the prefix/packages directory.
pub fn get_storage_packages(config: &Config) -> Result<HashSet<PackageId>> {
    let package_directory = config.prefix_directory.join("packages");
    let mut packages = Vec::new();
    for file_package in fs::read_dir(package_directory)? {
        let file_package = file_package?;
        if !file_package.path().is_dir() {
            continue;
        }

        // Get the package name
        let package_name = file_package.file_name();
        let package_name = package_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?;
        let package_name = PackageName::from_str(package_name)?;

        for file_version in fs::read_dir(file_package.path())? {
            let file_version = file_version?;
            if !file_version.path().is_dir() {
                continue;
            }

            // Get the version, and create the package id
            let version = Version::from_str(file_version.file_name().to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;
            let package_id = PackageId::new(package_name.clone(), version);

            packages.push(package_id);
        }
    }

    Ok(packages.into_iter().collect())
}
