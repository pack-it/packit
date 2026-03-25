use std::collections::HashSet;

use crate::installer::types::OptionalPackageId;

/// Gets all the duplicate package arguments. Packages are duplicate if they have the same name and version
/// or if the version isn't specified on one of them, the package is also considered duplicate.
pub fn get_duplicates(packages: &Vec<OptionalPackageId>) -> HashSet<String> {
    let mut duplicates = HashSet::new();
    let mut seen = HashSet::new();
    for package in packages {
        if let Some(package_version) = package.versioned() {
            if !seen.insert(package_version) {
                duplicates.insert(package.to_string());
            }

            continue;
        }

        // Check if any of the packages have the same name.
        // Also make sure not to match on the same package item in the vec by checking the pointer.
        if packages.iter().any(|p| !std::ptr::eq(p, package) && p.name == package.name) {
            duplicates.insert(package.name.to_string());
        }
    }

    duplicates
}
