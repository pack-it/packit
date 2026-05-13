// SPDX-License-Identifier: GPL-3.0-onl
use std::process::exit;

use crate::{
    cli::display::logging::error,
    config::Config,
    installer::types::{PackageId, PackageName, Version},
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
    utils::{fuzzy, unwrap_or_exit::UnwrapOrExit},
};

/// Shows possible register versions and exits with status 1 if the given package name exits.
pub fn register_version(package_name: &PackageName, register: &PackageRegister) {
    // Return early if the package name doesn't exist in the register
    let Some(package) = register.get_package(package_name) else {
        return;
    };

    display_versions(package.versions.keys())
}

/// Shows possible repository versions and exits with status 1 if the given package name exits.
pub fn repository_version(package_name: &PackageName, manager: &RepositoryManager) {
    // Return early if the package name doesn't exist in the repository
    let Ok((_, package)) = manager.read_package(package_name) else {
        return;
    };

    display_versions(package.versions.iter())
}

/// Displays alternative versions and exits.
fn display_versions<'a>(versions: impl Iterator<Item = &'a Version>) -> ! {
    print!("Did you mean version(s): ");
    for version in versions {
        print!("'{version}' ");
    }
    println!();

    exit(1);
}

/// Shows an error that the package (name) cannot be found (in the register) and a fuzzy alternative. Then exits at the end.
pub fn register_package(package_name: &PackageName, register: &PackageRegister) -> ! {
    error!(msg: "Package '{}' cannot be found", package_name);

    let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), package_name);
    if let Some(fuzzy_match) = fuzzy_match {
        println!("Did you mean: '{fuzzy_match}'?");
    }

    exit(1);
}

/// Shows an error that the package (name) cannot be found (in the repository) and a fuzzy alternative. Then exits at the end.
pub fn repository_package(package_name: &PackageName, manager: &RepositoryManager, config: &Config) -> ! {
    error!(msg: "Package '{package_name}' cannot be found.");

    let fuzzy_match = fuzzy::repository_search(config, manager, package_name).unwrap_or_exit(1);
    if let Some(fuzzy_match) = fuzzy_match {
        println!("Did you mean: '{fuzzy_match}'?");
    }

    exit(1);
}

/// Shows an error that the package version cannot be found (in the register) and a fuzzy alternative. Then exits at the end.
pub fn register_package_version(package_id: &PackageId, register: &PackageRegister) -> ! {
    error!(msg: "Package '{package_id}' cannot be found.");
    register_version(&package_id.name, register);

    let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), &package_id.name);
    if let Some(fuzzy_match) = fuzzy_match {
        println!("Did you mean: '{fuzzy_match}'?");
    }

    exit(1);
}

/// Shows an error that the package version cannot be found (in the repository) and a fuzzy alternative. Then exits at the end.
pub fn repository_package_version(package_id: &PackageId, manager: &RepositoryManager, config: &Config) -> ! {
    error!(msg: "Package '{}' cannot be found.", package_id);
    repository_version(&package_id.name, manager);

    let fuzzy_match = fuzzy::repository_search(config, manager, &package_id.name).unwrap_or_exit(1);
    if let Some(fuzzy_match) = fuzzy_match {
        println!("Did you mean: '{fuzzy_match}'?");
    }

    exit(1);
}
