use clap::Args;

use crate::{
    cli::error::CommandError, config::Config, installed_packages::InstalledPackageStorage, installer::installer::Installer,
    repositories::manager::RepositoryManager,
};

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// The name of the package to uninstall
    pub package_name: String,

    /// The version of the package to uninstall
    #[arg(short, long)]
    pub version: Option<String>,
}

impl UninstallArgs {
    pub fn handle(
        &self,
        config: &Config,
        installed_storage: &mut InstalledPackageStorage,
        manager: &RepositoryManager,
    ) -> Result<(), CommandError> {
        Installer::new(config, installed_storage, manager).uninstall(&self.package_name, &self.version)?;

        Ok(())
    }
}
