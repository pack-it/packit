// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::{commands::HandleCommand, parameter_checks},
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
    pub packages: Vec<OptionalPackageId>,
}

const ISSUE_FOUND_MESSAGE: &str = "Consider running `pit fix` to resolve the issues above.";
const NO_ISSUE_FOUND_MESSAGE: &str = "No issues were found!";

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

        // Get the package ids
        let package_ids = parameter_checks::expand_optional_ids(&register, &config, &self.packages);

        // Check all packages if no packages are specified
        let packages: &Vec<PackageId> = match package_ids.is_empty() {
            true => &register.iterate_all().map(|p| p.package_id.clone()).collect(),
            false => &package_ids,
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
