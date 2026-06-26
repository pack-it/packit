// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{QuestionResponse, ask_user, logging::error},
    },
    config::Config,
    installer::types::{OptionalPackageId, PackageId},
    integrity::{Repairer, Verifier},
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
    utils::{parameter_checks, unwrap_or_exit::UnwrapOrExit},
};

/// Fixes all issues found by the check command. The user will be asked if they want to fix an issue for each issue type.
#[derive(Args, Debug)]
pub struct FixArgs {
    /// A list of packages to fix. Could be empty, then all packages are fixed
    pub packages: Vec<OptionalPackageId>,
}

const FIX_MESSAGE: &str = "Would you like to automatically apply the fix above?";
const UNSUCCESSFUL_FIX_MESSAGE: &str = "The issue could not be fixed, due to unknown reasons";

impl HandleCommand for FixArgs {
    fn handle(&self) {
        let mut verifier = Verifier::new();
        let mut repairer = Repairer::new();

        self.fix_initial(&mut verifier, &mut repairer);
        self.fix(&mut verifier, &mut repairer);
    }
}

impl FixArgs {
    /// Fixes the initial issues, which check basic files that Packit requires to run properly (for example Config.toml or Register.toml).
    fn fix_initial(&self, verifier: &mut Verifier, repairer: &mut Repairer) {
        let mut issue_index = -1;
        while let Some(issue) = verifier.next_initial_issue().unwrap_or_exit(1) {
            // Check if the index is the same as the previously found issue (meaning the same issue is found and the fix didn't work)
            if verifier.get_initial_check_index() as i32 - 1 == issue_index {
                error!(msg: UNSUCCESSFUL_FIX_MESSAGE);
                exit(1);
            }

            println!("{issue}");
            println!("{}", issue.get_fix_message());
            if ask_user(FIX_MESSAGE, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                exit(0);
            }

            // Repair the found issues
            repairer.fix_initial_issues(issue).unwrap_or_exit(1);

            // Reverse the verifier to redo the check to make sure the fix worked
            issue_index = verifier.reverse_initial_check() as i32;
        }
    }

    /// Does the normal fixes.
    fn fix(&self, verifier: &mut Verifier, repairer: &mut Repairer) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Get the package ids
        let package_ids = parameter_checks::expand_optional_ids(&register, &config, &self.packages);

        // Fix all packages if no packages are specified
        let packages: &Vec<PackageId> = match package_ids.is_empty() {
            true => &register.iterate_all().map(|p| p.package_id.clone()).collect(),
            false => &package_ids,
        };

        // Retrieve and fix the issues one by one
        let mut issue_index = -1;
        while let Some(issue) = verifier.next_issue(packages, &register, &config).unwrap_or_exit(1) {
            // Check if the index is the same as the previously found issue (meaning the same issue is found and the fix didn't work)
            if verifier.get_check_index() as i32 - 1 == issue_index {
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
            issue_index = verifier.reverse_check() as i32;
        }

        if !verifier.issues_found() {
            println!("No issues were found");
        }
    }
}
