// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, installer::types::PackageId, storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit, verifier::Verifier,
};

/// Checks for any inconsistencies or mistakes in the installed packages or in the Packit files itself.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// A vec of packages to check. Could be empty, then all packages are checked.
    pub packages: Vec<PackageId>,
}

impl HandleCommand for CheckArgs {
    fn handle(&self) {
        match self.packages.is_empty() {
            true => self.check_all(),
            false => self.check_packages(),
        }
    }
}

impl CheckArgs {
    /// Checks all packages.
    fn check_all(&self) {
        let mut verifier = Verifier::new();
        while let Some(issue) = verifier.next_initial_issue().unwrap_or_exit(1) {
            println!("{issue}")
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        while let Some(issue) = verifier.next_issue(&register, &config).unwrap_or_exit(1) {
            println!("{issue}")
        }

        // Return correct message based on found issues
        if verifier.issues_found() {
            println!("Consider running `pit fix` to resolve the issues above.");
        } else {
            println!("No issues were found");
        }
    }

    /// Checks the packages specified by the user.
    fn check_packages(&self) {
        let mut verifier = Verifier::new();
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        for package_id in &self.packages {
            while let Some(issue) = verifier.next_package_issue(package_id, &register, &config).unwrap_or_exit(1) {
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
