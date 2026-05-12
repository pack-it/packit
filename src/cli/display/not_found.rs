use std::process::exit;

use crate::{
    cli::display::logging::error,
    installer::types::{PackageId, PackageName},
    storage::package_register::PackageRegister,
    utils::fuzzy,
};

/// Shows possible versions and exits with status 1 if the given package name exits.
pub fn version_not_found(package_name: &PackageName, register: &PackageRegister) {
    // Return early if the package name doesn't exist
    let Some(package_name) = register.get_package(package_name) else {
        return;
    };

    let versions = package_name.versions.keys();
    print!("Did you mean version(s): ");
    for version in versions {
        print!("'{version}' ");
    }
    println!();

    exit(1);
}

/// Shows a package cannot be found error and a fuzzy alternative for a package name. Then exits at the end.
pub fn package_not_found(package_name: &PackageName, register: &PackageRegister) -> ! {
    error!(msg: "Package '{}' cannot be found", package_name);

    let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), package_name);
    if let Some(fuzzy_match) = fuzzy_match {
        println!("Did you mean: '{fuzzy_match}'?");
    }

    exit(1)
}

/// Shows a package cannot be found error and a fuzzy alternative for a package version. Then exits at the end.
pub fn package_version_not_found(package_id: &PackageId, register: &PackageRegister) -> ! {
    error!(msg: "Package '{package_id}' cannot be found.");
    version_not_found(&package_id.name, register);

    let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), &package_id.name);
    if let Some(fuzzy_match) = fuzzy_match {
        println!("Did you mean: '{fuzzy_match}'?");
    }

    exit(1);
}
