use clap::Args;

use crate::{
    cli::error::CommandError, config::Config, installed_packages::InstalledPackageStorage, installer::installer::Installer,
    repositories::manager::RepositoryManager,
};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The name of the package to install
    pub package_name: String,

    /// The version of the package to install
    #[arg(short, long)]
    pub version: Option<String>,
}

impl InstallArgs {
    pub fn handle(
        &self,
        config: &Config,
        installed_storage: &mut InstalledPackageStorage,
        manager: &RepositoryManager,
    ) -> Result<(), CommandError> {
        if installed_storage.get_package_versions(&self.package_name).len() >= 1 {
            println!("Package '{}' already exists.", &self.package_name);
        } else {
            Installer::new(&config, installed_storage, &manager).install(&self.package_name, &self.version)?
        }

        Ok(())
    }
}
