use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    installer::{Symlinker, types::PackageName},
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Unlinks the specified package, causing the package to be unavailable from the PATH environment variable.
#[derive(Args, Debug)]
pub struct UnlinkArgs {
    /// The name of the package to unlink
    pub package_name: PackageName,
}

impl HandleCommand for UnlinkArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_path = PackageRegister::get_default_path(&config);
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
        Symlinker::new(&config)
            .unlink_package(&mut register, &self.package_name)
            .unwrap_or_exit_msg("Unable to unlink package", 1);

        println!("Successfully unlinked {}", self.package_name);

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);
    }
}
