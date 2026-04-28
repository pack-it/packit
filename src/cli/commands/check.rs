// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, installer::types::PackageId, storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit, verifier::Verifier,
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
            // TODO: Don't throw an error when some issues are already found (not only here, but for every call to the verifier)
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
