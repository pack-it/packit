// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf, process::exit};

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error, not_found},
    },
    config::Config,
    installer::types::PackageId,
    packager,
    register::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Packages the specified package into a prebuild and store it in the destination directory, together with a checksum of the prebuild.
#[derive(Args, Debug)]
pub struct PackageArgs {
    /// Destination of the compressed package
    pub destination: PathBuf,

    /// The ids of the packages to package
    pub packages: Vec<PackageId>,

    /// True to sort the package into a prebuild directory
    #[arg(short, long, default_value = "false")]
    pub sorted: bool,

    /// True to package all installed packages
    #[arg(short, long, default_value = "false")]
    pub all: bool,
}

impl HandleCommand for PackageArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let packages: Vec<&PackageId> = match self.all {
            true => register.iterate_all().map(|p| &p.package_id).collect(),
            false => self.packages.iter().collect(),
        };

        if packages.is_empty() {
            error!(msg: "Nothing packaged, no packages specified");
            exit(1);
        }

        for package_id in &packages {
            // Get the correct install directory
            let destination = match self.sorted {
                true => {
                    let prefix = package_id.name.get_prefix().to_string();
                    &self.destination.join("packages").join(prefix).join(&package_id.name).join(package_id.version.to_string())
                },
                false => &self.destination,
            };

            self.package(package_id, destination, &config, &register);
        }
    }
}

impl PackageArgs {
    fn package(&self, package_id: &PackageId, destination: &PathBuf, config: &Config, register: &PackageRegister) {
        let package_version = match register.get_package_version(package_id) {
            Some(package_version) => package_version,
            None => not_found::register_package_version(package_id, register),
        };

        // Automatically create the destination directory
        fs::create_dir_all(destination).unwrap_or_exit_msg("Failed to create prebuild directory", 1);

        let spinner = Spinner::new();
        let spinner_message = format!("Packaging {package_id}");
        spinner.show(spinner_message.clone());

        packager::package(config, package_id, destination, package_version.revisions.len() as u64).unwrap_or_exit(1);

        spinner.finish(format!("{spinner_message} successful"));
        println!("Successfully packaged {package_id} to {:?}", destination);
    }
}
