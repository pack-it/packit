use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, installer::types::PackageId, repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister, utils::unwrap_or_exit::UnwrapOrExit, verifier::Verifier,
};

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Optional package id, to limit the check to the specified package
    package: Option<PackageId>,
}

impl HandleCommand for CheckArgs {
    // TODO: We can never verify Packit files, because they are loaded before the command execution
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);
        let mut verifier = Verifier::new(config);

        // Get all issues
        let error_message = "An error occured during the check, this error could be caused by one of the issues above and might still be fixed by `pit fix`. It's possible that not all issues were found.";
        match &self.package {
            Some(id) => {
                while let Some(issue) = verifier.next_package_issue(id, &register).unwrap_or_exit_msg(error_message, 1) {
                    print!("{issue}\n")
                }
            },
            None => {
                while let Some(issue) = verifier.next_issue(&register).unwrap_or_exit_msg(error_message, 1) {
                    print!("{issue}\n")
                }
            },
        }

        // Return correct message based on found issues
        if verifier.issues_found() {
            println!("Consider running `pit fix` to resolve the issues above.");
        } else {
            println!("No issues were found");
        }
    }
}
