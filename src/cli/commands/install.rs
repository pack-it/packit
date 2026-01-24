use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    error_handling::HandleError,
    installed_packages::InstalledPackageStorage,
    installer::{installer::Installer, types::Version},
    repositories::manager::RepositoryManager,
};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The name of the package to install
    pub package_name: String,

    /// The version of the package to install
    #[arg(short, long)]
    pub version: Option<Version>,
}

impl HandleCommand for InstallArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let installed_dir = InstalledPackageStorage::get_default_path();
        let mut installed_storage = InstalledPackageStorage::from(&installed_dir).unwrap_or_exit();

        // Make sure the package doesn't already exist before installing
        if installed_storage.get_package_versions(&self.package_name).len() >= 1 {
            println!("Package '{}' already exists.", &self.package_name);
            return;
        }

        Installer::new(&config, &mut installed_storage, &manager).install(&self.package_name, &self.version).unwrap_or_exit();

        // Save changes
        installed_storage.save_to(&installed_dir).unwrap_or_exit();
    }
}
