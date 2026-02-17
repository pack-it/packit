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

        // Show all issues
        let result = match &self.package {
            Some(id) => verifier.show_all_package_issues(id, &register),
            None => verifier.show_all_issues(&register),
        };

        // Return early with message if no issues were found
        if verifier.issues_found() {
            println!("No issues were found");
            return;
        }

        println!("Consider running `pit fix` to resolve the issues above.");

        // Handle the result
        result.unwrap_or_exit_msg("An error occured during the check, this error could be caused by one of the issues above and might still be fixed by `pit fix`. It's possible that not all issues were found.", 1);
    }
}
