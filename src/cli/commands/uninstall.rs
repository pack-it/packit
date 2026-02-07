use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    installer::{types::Version, Installer, InstallerOptions},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// The name of the package to uninstall
    pub package_name: String,

    /// The version of the package to uninstall
    #[arg(short, long)]
    pub version: Option<Version>,
}

impl HandleCommand for UninstallArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        Installer::new(config, &mut register, manager, InstallerOptions::default())
            .uninstall(&self.package_name, self.version.as_ref())
            .unwrap_or_exit(1);

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
