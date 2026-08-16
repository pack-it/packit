// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use std::process::exit;

use crate::{
    cli::{
        commands::HandleCommand,
        display::logging::{debug, error},
        parameter_checks,
    },
    config::Config,
    installer::types::{OptionalPackageId, PackageId},
    integrity::Verifier,
    register::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Checks for any inconsistencies or mistakes in the installed packages or in the Packit files itself.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// A list of packages to check. Could be empty, then all packages are checked
    packages: Vec<OptionalPackageId>,
}

const ISSUE_FOUND_MESSAGE: &str = "Consider running `pit fix` to resolve the issues above";

impl HandleCommand for CheckArgs {
    fn handle(&self) {
        // Always do initial checks first
        let mut verifier = Verifier::new();
        while verifier.get_initial_check_index() < verifier.get_initial_check_length() {
            match verifier.next_initial_check() {
                Ok(Some(issue)) => print!("{issue}"),
                Ok(_) => {},
                Err(e) if verifier.issues_found() > 0 => {
                    debug!(err: e, "An error occured when issues were already found, skipping remaining checks");
                    return;
                },
                Err(e) => {
                    error!(e, "An error occured while doing the initial verifier checks");
                    exit(1);
                },
            }
        }

        // Return correct message based on found issues
        if verifier.issues_found() > 0 {
            println!("{ISSUE_FOUND_MESSAGE}");
            return;
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Get the package ids
        let package_ids = parameter_checks::expand_optional_ids(&register, &config, &self.packages);

        // Check all packages if no packages are specified
        let packages: &Vec<PackageId> = match package_ids.is_empty() {
            true => &register.iterate_all().map(|p| p.package_id.clone()).collect(),
            false => &package_ids,
        };

        while verifier.get_check_index() < verifier.get_check_length() {
            match verifier.next_check(packages, &register, &config) {
                Ok(Some(issue)) => print!("{issue}"),
                Ok(_) => {},
                Err(e) if verifier.issues_found() > 0 => {
                    debug!(err: e, "An error occured when issues were already found, skipping remaining checks");
                    return;
                },
                Err(e) => {
                    error!(e, "An error occured while doing the verifier checks");
                    exit(1);
                },
            }
        }

        // Return correct message based on found issues
        if verifier.issues_found() > 0 {
            println!("{ISSUE_FOUND_MESSAGE}");
        } else {
            println!("No issues were found!");
        }
    }
}
