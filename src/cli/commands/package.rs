// SPDX-License-Identifier: GPL-3.0-only
use std::{path::PathBuf, process::exit};

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error},
    },
    config::Config,
    installer::types::PackageId,
    packager::{self},
    storage::package_register::PackageRegister,
    utils::{fuzzy::min_fuzzy_search, unwrap_or_exit::UnwrapOrExit},
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
                error!(msg: "Package '{}' cannot be found.", self.package_id);

                // Show possible versions if a package with the given name exists
                if let Some(package_name) = register.get_package(&self.package_id.name) {
                    let versions = package_name.versions.keys();
                    print!("Did you mean version(s): ");
                    for version in versions {
                        print!("'{version}' ");
                    }
                    println!();
                    return;
                }

                let fuzzy_match = min_fuzzy_search(register.iterate_package_names(), &self.package_id.name);
                if let Some(fuzzy_match) = fuzzy_match {
                    println!("Did you mean: '{fuzzy_match}'?");
                }

                exit(1);
            },
        };

        let spinner = Spinner::new();
        let spinner_message = format!("Packaging {}", self.package_id);
        spinner.show(spinner_message.clone());

        packager::package(&config, &self.package_id, &self.destination, package_version.revisions.len()).unwrap_or_exit(1);

        spinner.finish(format!("{spinner_message} successful"));
        println!("Successfully packaged {} to {:?}", self.package_id, self.destination);
    }
}
