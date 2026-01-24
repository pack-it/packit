use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, error_handling::HandleError, installed_packages::InstalledPackageStorage,
    repositories::manager::RepositoryManager, verifier::get_packages,
};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Flag to indicate a full check (actually check packit install directory)
    #[arg(short, long)]
    use_dir: bool,
}

impl HandleCommand for ListArgs {
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let installed_dir = InstalledPackageStorage::get_default_path();
        let installed_storage = InstalledPackageStorage::from(&installed_dir).unwrap_or_exit();

        if self.use_dir {
            for package in get_packages(&config).unwrap_or_exit() {
                println!("{}", package);
            }
        } else {
            for package in &installed_storage.installed_packages {
                println!("{} {}", package.name, package.version);
            }
        }
    }
}
