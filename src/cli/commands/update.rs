// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{
        Installer, InstallerOptions,
        types::{OptionalPackageId, Version},
    },
    platforms::Target,
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::{fuzzy, unwrap_or_exit::UnwrapOrExit},
};

/// Updates the specified package to the new version, or the latest version if no new version is specified.
/// If multiple versions of the same package are installed, the <VERSION> option is required.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// The name of the package to update, with an optional version specified with NAME@VERSION.
    optional_id: OptionalPackageId,

    /// The version to update to. This can only be a higher version than the current version.
    new_version: Option<Version>,
}

impl HandleCommand for UpdateArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        if let Some(package_id) = self.optional_id.versioned() {
            if register.get_package_version(&package_id).is_none() {
                error!(msg: "Package '{}' cannot be found.", package_id);

                // Show possible versions if a package with the given name exists
                if let Some(package) = register.get_package(&package_id.name) {
                    let versions = package.versions.keys();
                    print!("Did you mean version(s): ");
                    for version in versions {
                        print!("'{version}' ");
                    }
                    println!();
                    return;
                }
            }
        }

        if register.get_package(&self.optional_id.name).is_none() {
            error!(msg: "Package '{}' cannot be found.", self.optional_id.name);

            let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), &self.optional_id.name);
            if let Some(fuzzy_match) = fuzzy_match {
                println!("Did you mean: '{fuzzy_match}'?");
            }

            return;
        }

        let (_, package_meta) = manager.read_package(&self.optional_id.name).unwrap_or_exit(1);

        let new_version = match &self.new_version {
            Some(version) => version,
            None => package_meta.get_latest_version(&Target::current()).unwrap_or_exit(1),
        };

        // Check if new version exists
        if !package_meta.versions.contains(new_version) {
            error!(msg: "New package version '{new_version}' does not exist.");

            // Show possible versions if a package with the given name exists
            if let Ok((_, package_name)) = manager.read_package(&self.optional_id.name) {
                let versions = package_name.versions;
                print!("Did you mean version(s): ");
                for version in versions {
                    print!("'{version}' ");
                }
                println!();
                return;
            }

            exit(1);
        }

        let options = InstallerOptions::default().skip_active(true).skip_symlinking(true);
        let mut installer = Installer::new(&config, &mut register, &manager, options);

        let new_package_id = installer.update(&self.optional_id, new_version).unwrap_or_exit(1);

        println!("Successfully updated {} to {new_package_id}", self.optional_id);

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
