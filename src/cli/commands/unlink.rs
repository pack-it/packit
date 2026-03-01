use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, installer::Symlinker, repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister, utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct UnlinkArgs {
    /// The name of the package to unlink
    pub package_name: String,
}

impl HandleCommand for UnlinkArgs {
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let register_path = PackageRegister::get_default_path(config);
        let mut register = PackageRegister::from(&register_path).unwrap_or_exit(1);

        // Get installed package
        let package = register
            .get_package(&self.package_name)
            .unwrap_or_exit_msg(&format!("Package {} is not installed!", self.package_name), 1);

        // Check if the package is already symlinked
        if !package.symlinked {
            println!("This package is currently not symlinked");
            return;
        }

        // Unlink package
        Symlinker::new(config)
            .unlink_package(&mut register, &self.package_name)
            .unwrap_or_exit_msg("Unable to unlink package", 1);

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);
    }
}
