use clap::Args;

use crate::{
    cli::commands::{CommandError, HandleCommand},
    config::Config,
    installed_packages::InstalledPackageStorage,
    installer::installer::Installer,
    repositories::manager::RepositoryManager,
    version::Version,
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
    fn handle(&self, config: &Config, manager: &RepositoryManager) -> Result<(), CommandError> {
        let installed_dir = InstalledPackageStorage::get_default_path();
        let mut installed_storage = InstalledPackageStorage::from(&installed_dir)?;

        Installer::new(config, &mut installed_storage, manager).uninstall(&self.package_name, &self.version)?;

        // Save changes
        installed_storage.save_to(&installed_dir)?;

        Ok(())
    }
}
