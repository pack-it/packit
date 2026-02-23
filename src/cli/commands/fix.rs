use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{ask_user, QuestionResponse},
    },
    config::Config,
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
    verifier::{Repairer, Verifier},
};

#[derive(Args, Debug)]
pub struct FixArgs;

impl HandleCommand for FixArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);
        let mut verifier = Verifier::new(config);

        let mut repairer = Repairer::new(config, manager);

        // Retrieve and fix the issues one by one
        while let Some(issue) = verifier.next_issue(&register).unwrap_or_exit(1) {
            print!("{issue}\n");

            let question = "Would you like to automatically fix the above issue with `pit fix`?";
            if ask_user(question, QuestionResponse::Yes).unwrap_or_exit(1).is_no() {
                continue;
            }

            // Repair the found issues
            repairer.fix(issue, &mut register).unwrap_or_exit(1);
        }

        // Return correct message based on found issues
        if !verifier.issues_found() {
            println!("No issues were found");
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
