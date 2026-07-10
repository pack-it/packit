// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs};

use crate::{
    cli::display::{not_found, styled::Styled},
    config::Config,
    installer::types::{OptionalPackageId, PackageId},
    register::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Gets all the duplicate package arguments. Packages are duplicate if they have the same name and version
/// or if the version isn't specified on one of them, the package is also considered duplicate.
pub fn get_duplicates(packages: &Vec<OptionalPackageId>) -> HashSet<String> {
    let mut duplicates = HashSet::new();
    let mut seen = HashSet::new();
    for package in packages {
        if let Some(package_version) = package.versioned() {
            if !seen.insert(package_version) {
                duplicates.insert(package.style().to_string());
            }

            continue;
        }

        // Check if any of the packages have the same name.
        // Also make sure not to match on the same package item in the vec by checking the pointer.
        if packages.iter().any(|p| !std::ptr::eq(p, package) && p.name == package.name) {
            duplicates.insert(package.name.style().to_string());
        }
    }

    duplicates
}

/// Expands all the optional ids to package ids. Will exit with a not found error if the given optional id doesn't exist in the register.
/// Returns a list of `PackageId`.
pub fn expand_optional_ids(register: &PackageRegister, config: &Config, packages: &[OptionalPackageId]) -> Vec<PackageId> {
    let mut package_ids = Vec::new();
    for optional_id in packages {
        if let Some(package_id) = optional_id.versioned() {
            let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
            if register.get_package_version(&package_id).is_none() && !fs::exists(installed_directory).unwrap_or_exit(1) {
                not_found::register_package_version(&package_id, register);
            }

            package_ids.push(package_id);
            continue;
        }

        let installed_directory = config.prefix_directory.join("packages").join(&optional_id.name);
        if register.get_package(&optional_id.name).is_none() && !fs::exists(installed_directory).unwrap_or_exit(1) {
            not_found::register_package(&optional_id.name, register);
        }

        let package = register.get_package(&optional_id.name).unwrap();
        package.versions.keys().for_each(|v| package_ids.push(PackageId::new(optional_id.name.clone(), v.clone())));
    }

    package_ids
}
