// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, process::exit};

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::PackageId,
    storage::package_register::PackageRegister,
    utils::{fuzzy, unwrap_or_exit::UnwrapOrExit},
    verifier::Verifier,
};

/// Checks for any inconsistencies or mistakes in the installed packages or in the Packit files itself.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// A vec of packages to fix. Could be empty, then all packages are checked.
    pub packages: Vec<PackageId>,
}

impl HandleCommand for CheckArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);
        let mut verifier = Verifier::new(&config);

        let error_message = "An error occured during the check, this error could be caused by one of the issues above and might still be fixed by `pit fix`. It's possible that not all issues were found (especially when checking a single package).";
        if self.packages.is_empty() {
            while let Some(issue) = verifier.next_issue(&register).unwrap_or_exit_msg(error_message, 1) {
                println!("{issue}")
            }

            // Return correct message based on found issues
            if verifier.issues_found() {
                println!("Consider running `pit fix` to resolve the issues above.");
            } else {
                println!("No issues were found");
            }

            return;
        }

        for package_id in &self.packages {
            // Check if the package exists in the register or in storage before doing any checks
            // TODO: The verifier should handle this (at the places where it currently says "note doesn't check for existance in xyz")
            let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
            if register.get_package_version(package_id).is_none() && !fs::exists(installed_directory).unwrap_or_exit(1) {
                error!(msg: "Cannot perform checks, package '{package_id}' doesn't exist in register or storage.");

                // Show possible versions if a package with the given name exists
                if let Some(package_name) = register.get_package(&package_id.name) {
                    print!("Did you mean version(s): ");
                    let versions = package_name.versions.keys();
                    for version in versions {
                        print!("'{version}' ");
                    }

                    println!();
                    return;
                }

                let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), &package_id.name);
                if let Some(fuzzy_match) = fuzzy_match {
                    println!("Did you mean: '{fuzzy_match}'?");
                }

                exit(1);
            }

            while let Some(issue) = verifier.next_package_issue(package_id, &register).unwrap_or_exit_msg(error_message, 1) {
                println!("{issue}")
            }

            // Return correct message based on found issues
            if verifier.issues_found() {
                println!("Consider running `pit fix` to resolve the issues above.");
            } else {
                println!("No issues were found");
            }
        }
    }
}
