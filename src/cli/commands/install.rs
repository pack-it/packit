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
        let mut installed_storage = InstalledPackageStorage::from(&installed_dir).unwrap_or_exit(1);

        // TODO: Check if this exists as an external package (possibly leading to conflicts) (if so, add to external packages)

        Installer::new(&config, &mut installed_storage, &manager)
            .install(&self.package_name, self.version.as_ref())
            .unwrap_or_exit(1);

        // Save changes
        installed_storage.save_to(&installed_dir).unwrap_or_exit(1);
    }
}
