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
pub struct InstallArgs {
    /// The name of the package to install
    pub package_name: String,

    /// The version of the package to install
    #[arg(short, long)]
    pub version: Option<Version>,
}

impl HandleCommand for InstallArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) -> Result<(), CommandError> {
        let installed_dir = InstalledPackageStorage::get_default_path();
        let mut installed_storage = InstalledPackageStorage::from(&installed_dir)?;

        if installed_storage.get_package_versions(&self.package_name).len() >= 1 {
            println!("Package '{}' already exists.", &self.package_name);
        } else {
            Installer::new(&config, &mut installed_storage, &manager).install(&self.package_name, &self.version)?
        }

        // Save changes
        installed_storage.save_to(&installed_dir)?;

        Ok(())
    }
}
