// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use std::process::exit;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{QuestionResponse, ask_user, logging::error},
    },
    installer::types::OptionalPackageId,
    integrity::Verifier,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Fixes all issues found by the check command. The user will be asked if they want to fix an issue for each issue type.
#[derive(Args, Debug)]
pub struct FixArgs {
    /// A list of packages to fix. Could be empty, then all packages are fixed
    packages: Vec<OptionalPackageId>,
}

impl HandleCommand for FixArgs {
    fn handle(&self) {
        let mut verifier = Verifier::new(self.packages.clone());

        // Retrieve and fix the issues one by one
        let mut fixed_issue = false;
        while verifier.get_check_index() < verifier.get_check_length() {
            let issue = match verifier.next_check() {
                // If there is an issue and we previously attempted to fix it, error and exit
                Ok(Some(_)) if fixed_issue => {
                    error!(msg: "The issue could not be fixed, due to unknown reasons");
                    exit(1);
                },

                // If there is an issue, return the issue to fix it
                Ok(Some(issue)) => issue,

                // If there is no issue and we previously attempted to fix it, show a success message and reset `fixed_issue`
                Ok(None) if fixed_issue => {
                    println!("{}", "Successfully fixed the issue".bold().green());
                    fixed_issue = false;
                    continue;
                },

                // If there is no issue, continue
                Ok(None) => continue,

                // Display errors and exit
                Err(e) => {
                    error!(e, "An error occurred while doing the verifier checks");
                    exit(1);
                },
            };

            println!("{issue}");
            println!("{}", issue.get_fix_message());
            let question = "Would you like to automatically apply the fix above?";
            if ask_user(question, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                println!("The issue has NOT been fixed and no further checks were done");
                return;
            }

            // Repair the found issues
            if let Err(e) = verifier.fix(issue) {
                error!(e, "An error occurred while trying to fix the issue");
                exit(1);
            }

            // Reverse the verifier to redo the check to make sure the fix worked
            verifier.reverse_check();

            fixed_issue = true;
        }

        if verifier.issues_found() == 0 {
            println!("No issues were found!");
        }
    }
}
