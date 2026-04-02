// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{QuestionResponse, ask_user},
    },
    config::Config,
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
    verifier::{Repairer, Verifier},
};

/// Fixes all issues found by the check command. The user will be asked if they want to fix an issue for each issue type.
#[derive(Args, Debug)]
pub struct FixArgs;

impl HandleCommand for FixArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);
        let mut verifier = Verifier::new(&config);

        let mut repairer = Repairer::new(&config, &manager);

        // Retrieve and fix the issues one by one
        while let Some(issue) = verifier.next_issue(&register).unwrap_or_exit(1) {
            print!("{issue}\n");

            let question = "Would you like to automatically fix the above issue with `pit fix`?";
            if ask_user(question, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                continue;
            }

            // Repair the found issues
            repairer.fix(issue, &mut register).unwrap_or_exit(1);

            // Save register after the fix
            register.save_to(&register_dir).unwrap_or_exit(1);
        }

        // Return correct message based on found issues
        if !verifier.issues_found() {
            println!("No issues were found");
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
