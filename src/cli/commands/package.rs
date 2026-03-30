use std::{path::PathBuf, process::exit};

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::PackageId,
    packager::{self},
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Packages the specified package into a prebuild and store it in the destination directory, together with a checksum of the prebuild.
#[derive(Args, Debug)]
pub struct PackageArgs {
    /// The package id of the package
    pub package_id: PackageId,

    /// Destination of the compressed package
    pub destination: PathBuf,
}

impl HandleCommand for PackageArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let package_version = match register.get_package_version(&self.package_id) {
            Some(package_version) => package_version,
            None => {
                error!(msg: "Cannot package package, because the given package does not exist.");
                exit(1);
            },
        };

        packager::package(&config, &self.package_id, &self.destination, package_version.revisions.len()).unwrap_or_exit(1);

        println!("Successfully packaged {} to {:?}", self.package_id, self.destination);
    }
}
