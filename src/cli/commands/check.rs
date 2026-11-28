// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use std::process::exit;

use crate::{
    cli::{
        commands::HandleCommand,
        display::logging::{debug, error},
    },
    installer::types::OptionalPackageId,
    integrity::Verifier,
};

/// Checks for any inconsistencies or mistakes in the installed packages or in the Packit files itself.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// A list of packages to check. Could be empty, then all packages are checked
    packages: Vec<OptionalPackageId>,
}

impl HandleCommand for CheckArgs {
    fn handle(&self) {
        let mut verifier = Verifier::new(self.packages.clone());
        while verifier.get_check_index() < verifier.get_check_length() {
            match verifier.next_check() {
                Ok(Some(issue)) => print!("{issue}"),
                Ok(_) => {},
                Err(e) if verifier.issues_found() > 0 => {
                    debug!(err: e, "An error occurred when issues were already found, skipping remaining checks");
                    return;
                },
                Err(e) => {
                    error!(e, "An error occurred while doing the verifier checks");
                    exit(1);
                },
            }
        }

        // Return correct message based on found issues
        if verifier.issues_found() > 0 {
            println!("Some issues were found, the most urgent issues are listed first.");
            println!("Consider running `pit fix` to resolve the issues above.");
        } else {
            println!("No issues were found!");
        }
    }
}
