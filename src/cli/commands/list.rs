use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, error_handling::HandleError, repositories::manager::RepositoryManager,
    storage::installed_packages::InstalledPackageStorage, verifier::get_packages,
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
        let installed_storage = InstalledPackageStorage::from(&installed_dir).unwrap_or_exit(1);

        if self.use_dir {
            for package in get_packages(&config).unwrap_or_exit(1) {
                println!("{}", package);
            }
        } else {
            for package in &installed_storage.installed_packages {
                println!("{}", package.package_id);
            }
        }
    }
}
