// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{QuestionResponse, ask_user, logging::error},
    },
    config::Config,
    installer::types::PackageId,
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
    verifier::{Repairer, Verifier},
};

/// Fixes all issues found by the check command. The user will be asked if they want to fix an issue for each issue type.
#[derive(Args, Debug)]
pub struct FixArgs {
    /// A vec of packages to fix. Could be empty, then all packages are fixed.
    pub packages: Vec<PackageId>,
}

const FIX_MESSAGE: &str = "Would you like to automatically apply the fix above?";
const UNSUCCESSFUL_FIX_MESSAGE: &str = "The issue could not be fixed, due to unknown reasons";

impl HandleCommand for FixArgs {
    fn handle(&self) {
        let mut verifier = Verifier::new();
        let mut repairer = Repairer::new();

        match self.packages.is_empty() {
            true => self.fix_all(&mut verifier, &mut repairer),
            false => self.fix_packages(&mut verifier, &mut repairer),
        }
    }
}

impl FixArgs {
    /// Fixes all packages.
    fn fix_all(&self, verifier: &mut Verifier, repairer: &mut Repairer) {
        let mut issue_index = -1;
        while let Some(issue) = verifier.next_initial_issue().unwrap_or_exit(1) {
            if verifier.get_initial_check_index() as i32 - 1 == issue_index {
                error!(msg: UNSUCCESSFUL_FIX_MESSAGE);
                exit(1);
            }

            println!("{issue}");
            println!("{}", issue.get_fix_message());
            if ask_user(FIX_MESSAGE, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                return;
            }

            // Repair the found issues
            repairer.fix_initial_issues(issue).unwrap_or_exit(1);

            // Reverse the verifier to redo the check to make sure the fix worked
            issue_index = verifier.reverse_initial_check() as i32;
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Retrieve and fix the issues one by one
        let mut issue_index = -1;
        while let Some(issue) = verifier.next_issue(&register, &config).unwrap_or_exit(1) {
            if verifier.get_general_check_index() as i32 - 1 == issue_index {
                error!(msg: UNSUCCESSFUL_FIX_MESSAGE);
                exit(1);
            }

            println!("{issue}");
            println!("{}", issue.get_fix_message());
            if ask_user(FIX_MESSAGE, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                return;
            }

            // Repair the found issues
            repairer.fix(issue, &mut register, &config, &manager).unwrap_or_exit(1);

            // Save register after each fix
            register.save_to(&register_dir).unwrap_or_exit(1);

            // Reverse the verifier to redo the check to make sure the fix worked
            issue_index = verifier.reverse_general_check() as i32;
        }

        // Return correct message based on found issues
        if !verifier.issues_found() {
            println!("No issues were found");
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }

    /// Fixes the packages specified by the user.
    fn fix_packages(&self, verifier: &mut Verifier, repairer: &mut Repairer) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        for package_id in &self.packages {
            // Retrieve and fix the issues one by one
            let mut issue_index = -1;
            while let Some(issue) = verifier.next_package_issue(package_id, &register, &config).unwrap_or_exit(1) {
                if verifier.get_package_check_index() as i32 - 1 == issue_index {
                    error!(msg: UNSUCCESSFUL_FIX_MESSAGE);
                    exit(1);
                }

                println!("{issue}");
                println!("{}", issue.get_fix_message());
                if ask_user(FIX_MESSAGE, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                    return;
                }

                // Repair the found issues
                repairer.fix(issue, &mut register, &config, &manager).unwrap_or_exit(1);

                // Save register after each fix
                register.save_to(&register_dir).unwrap_or_exit(1);

                // Reverse the verifier to redo the check to make sure the fix worked
                issue_index = verifier.reverse_package_check() as i32;
            }

            // Show when no errors are found for the current package
            if !verifier.issues_found() {
                println!("No issues were found for {package_id}");
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
