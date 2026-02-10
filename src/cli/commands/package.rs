use std::{path::PathBuf, process::exit};

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::PackageId,
    packager::{self},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct PackageArgs {
    /// The package id of the package
    pub package_id: PackageId,

    /// Destination of the compressed package
    pub destination: PathBuf,
}

impl HandleCommand for PackageArgs {
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Throw an error if the specified package doesn't exist
        if register.get_package_version(&self.package_id).is_none() {}

        let package_version = match register.get_package_version(&self.package_id) {
            Some(package_version) => package_version,
            None => {
                error!(msg: "Cannot package package, because the given package does not exist.");
                exit(1);
            },
        };

        packager::package(config, &self.package_id, &self.destination, package_version.revisions.len()).unwrap_or_exit(1);
    }
}
