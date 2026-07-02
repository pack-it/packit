// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, not_found},
        parameter_checks,
    },
    config::Config,
    installer::{
        Installer, InstallerOptions,
        types::{OptionalPackageId, Version},
    },
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Updates the specified package to the new version, or the latest version if no new version is specified.
/// If multiple versions of the same package are installed, the <VERSION> option is required.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// The name of the package to update, with an optional version specified with NAME@VERSION
    optional_ids: Vec<OptionalPackageId>,

    /// The version to update to. This can only be a higher version than the current version and is only used when a single package is specified
    #[arg(long, requires = "optional_ids")]
    new_version: Option<Version>,
}

impl HandleCommand for UpdateArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        if self.optional_ids.len() > 1 && self.new_version.is_some() {
            error!(msg: "Cannot specify new version when multiple packages are given");
            exit(1);
        }

        // Check for duplicates, because updating twice will result in an error
        let duplicates = parameter_checks::get_duplicates(&self.optional_ids);
        if !duplicates.is_empty() {
            let mut duplicate_string = String::new();
            for duplicate in duplicates {
                duplicate_string.push_str(&duplicate.to_string());
                duplicate_string.push(' ');
            }

            error!(msg: "Duplicate package arguments are not allowed. The following duplicates were found: {duplicate_string}");
            exit(1);
        }

        for optional_id in &self.optional_ids {
            match optional_id.versioned() {
                Some(package_id) if register.get_package_version(&package_id).is_some() => {},
                Some(package_id) => not_found::register_package_version(&package_id, &register),
                None if register.get_package(&optional_id.name).is_some() => {},
                None => not_found::register_package(&optional_id.name, &register),
            }

            let (_, package_meta) = manager.read_package(&optional_id.name).unwrap_or_exit(1);

            // Note that there is a check earlier which checks if the new_version is `None` in case of multiple packages
            let new_version = match &self.new_version {
                Some(version) => version,
                None => package_meta.get_latest_version(&Target::current()).unwrap_or_exit(1),
            };

            // Check if the new version exists
            if !package_meta.versions.contains(new_version) {
                error!(msg: "New package version '{new_version}' does not exist.");
                not_found::repository_version(&optional_id.name, &manager);
                exit(1);
            }

            let options = InstallerOptions::default();
            let mut installer = Installer::new(&config, &mut register, &manager, options);

            // Do the update, and in case of an error throw the error, but continue
            let new_package_id = match installer.update(&optional_id, new_version) {
                Ok(new_package_id) => new_package_id,
                Err(error) => {
                    error!(error, "Cannot update package {optional_id}");
                    continue;
                },
            };

            println!("Successfully updated {} to {new_package_id}", optional_id);

            // Save changes
            register.save_to(&register_dir).unwrap_or_exit(1);
        }
    }
}
