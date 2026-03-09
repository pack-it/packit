use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{Installer, InstallerOptions, types::OptionalPackageId},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// The name of the packages to install, with an optional version specified with NAME@VERSION
    #[arg(num_args(0..))]
    pub packages: Vec<OptionalPackageId>,
}

impl HandleCommand for UninstallArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let mut installer = Installer::new(&config, &mut register, &manager, InstallerOptions::default());

        // Uninstall all specified packages
        for optional_id in &self.packages {
            if let Err(error) = installer.uninstall(&optional_id) {
                error!(error, "Cannot uninstall package {optional_id}");
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
