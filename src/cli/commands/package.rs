// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf};

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, not_found},
    },
    config::Config,
    installer::types::PackageId,
    packager,
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

    /// True to sort the package into a prebuild directory
    #[arg(short, long, default_value = "false")]
    pub sorted: bool,
}

impl HandleCommand for PackageArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let package_version = match register.get_package_version(&self.package_id) {
            Some(package_version) => package_version,
            None => not_found::register_package_version(&self.package_id, &register),
        };

        // Get the correct install directory
        let destination = match self.sorted {
            true => {
                let prefix = self.package_id.name.get_prefix().to_string();
                &self.destination.join("packages").join(prefix).join(&self.package_id.name).join(self.package_id.version.to_string())
            },
            false => &self.destination,
        };

        // Automatically create the destination directory
        fs::create_dir_all(destination).unwrap_or_exit_msg("Failed to create prebuild directory", 1);

        let spinner = Spinner::new();
        let spinner_message = format!("Packaging {}", self.package_id);
        spinner.show(spinner_message.clone());

        packager::package(&config, &self.package_id, destination, package_version.revisions.len() as u64).unwrap_or_exit(1);

        spinner.finish(format!("{spinner_message} successful"));
        println!("Successfully packaged {} to {:?}", self.package_id, self.destination);
    }
}
