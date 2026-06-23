// SPDX-License-Identifier: GPL-3.0-only
use std::fs;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::not_found},
    config::Config,
    installer::types::PackageId,
    integrity::Verifier,
    register::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Checks for any inconsistencies or mistakes in the installed packages or in the Packit files itself.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// A list of packages to check. Could be empty, then all packages are checked
    pub packages: Vec<PackageId>,
}

const ISSUE_FOUND_MESSAGE: &str = "Consider running `pit fix` to resolve the issues above.";
const NO_ISSUE_FOUND_MESSAGE: &str = "No issues were found";

impl HandleCommand for CheckArgs {
    fn handle(&self) {
        // Always do initial checks first
        let mut verifier = Verifier::new();
        while let Some(issue) = verifier.next_initial_issue().unwrap_or_exit(1) {
            println!("{issue}")
        }

        // Return correct message based on found issues
        if verifier.issues_found() {
            println!("{ISSUE_FOUND_MESSAGE}");
            return;
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Make sure all specified packages exist
        for package_id in &self.packages {
            let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
            if register.get_package_version(package_id).is_none() && !fs::exists(installed_directory).unwrap_or_exit(1) {
                not_found::register_package_version(package_id, &register);
            }
        }

        // Check all packages if no packages are specified
        let packages: &Vec<PackageId> = match self.packages.is_empty() {
            true => &register.iterate_all().map(|p| p.package_id.clone()).collect(),
            false => &self.packages,
        };

        while let Some(issue) = verifier.next_issue(&packages, &register, &config).unwrap_or_exit(1) {
            println!("{issue}")
        }

        // Return correct message based on found issues
        if verifier.issues_found() {
            println!("{ISSUE_FOUND_MESSAGE}");
        } else {
            println!("{NO_ISSUE_FOUND_MESSAGE}");
        }
    }
}
