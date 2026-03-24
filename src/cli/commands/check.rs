use std::{fs, process::exit};

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::PackageId,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
    verifier::Verifier,
};

/// The check command uses the verifier to check for any inconsistencies or mistakes
/// in the installed packages or in the Packit files itself.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Optional package id, to limit the check to the specified package
    package: Option<PackageId>,
}

impl HandleCommand for CheckArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);
        let mut verifier = Verifier::new(&config);

        // Check if the package exists before checking it with the verifier
        if let Some(package_id) = &self.package {
            // Check if the package exists in the register or in storage before doing any checks
            let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
            if register.get_package_version(package_id).is_none() && !fs::exists(installed_directory).unwrap_or_exit(1) {
                error!(msg: "Cannot perform checks, package {package_id} doesn't exist in register or storage.");
                exit(1);
            }
        }

        // Show all issues
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
