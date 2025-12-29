use std::path::PathBuf;

use clap::Args;

use crate::{
    config::Config,
    installed_packages::InstalledPackageStorage,
    verifier::{get_packages, VerifierError},
};

// TODO: Maybe rename to just List
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Directory to list all packages of (OPTIONAL)
    directory: Option<PathBuf>, // TODO: Unused atm

    /// Flag to indicate a full check (actually check packit install directory)
    #[arg(short, long)]
    use_dir: bool,
}

impl ListArgs {
    /// Handles the list command with user specified arguments.
    pub fn handle(&self, installed_storage: &InstalledPackageStorage, config: &Config) -> Result<(), VerifierError> {
        if self.use_dir {
            for package in get_packages(&config)? {
                println!("{}", package);
            }
        } else {
            for package in &installed_storage.installed_packages {
                println!("{} {}", package.name, package.version);
            }
        }

        Ok(())
    }
}
