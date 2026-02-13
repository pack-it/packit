use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{types::OptionalPackageId, Installer, InstallerOptions},
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
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let mut installer = Installer::new(config, &mut register, manager, InstallerOptions::default());

        // Uninstall all specified packages
        for package_id in &self.packages {
            if let Err(error) = installer.uninstall(&package_id.name, package_id.version.as_ref()) {
                error!(error, "Cannot uninstall package {package_id}");
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
