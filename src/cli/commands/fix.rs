use clap::Args;

use crate::{
    cli::commands::HandleCommand,
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

        // Repair the found issues
        let mut repairer = Repairer::new(config, manager);
        repairer.fix(&mut verifier, &mut register).unwrap_or_exit(1);

        // Return correct message based on found issues
        if !verifier.issues_found() {
            println!("No issues were found");
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
