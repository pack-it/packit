use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    error_handling::HandleError,
    installer::{installer::Installer, types::Version},
    repositories::manager::RepositoryManager,
    storage::installed_packages::InstalledPackageStorage,
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
        let installed_dir = InstalledPackageStorage::get_default_path();
        let mut installed_storage = InstalledPackageStorage::from(&installed_dir).unwrap_or_exit(1);

        Installer::new(config, &mut installed_storage, manager).uninstall(&self.package_name, &self.version).unwrap_or_exit(1);

        // Save changes
        installed_storage.save_to(&installed_dir).unwrap_or_exit(1);
    }
}
