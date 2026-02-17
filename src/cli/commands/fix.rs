use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, repositories::manager::RepositoryManager, storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit, verifier::Repairer,
};

#[derive(Args, Debug)]
pub struct FixArgs;

impl HandleCommand for FixArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Repair the found issues
        let mut repairer = Repairer::new(config, manager);
        repairer.fix(&mut register).unwrap_or_exit(1);

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
