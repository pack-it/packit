// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{QuestionResponse, ask_user, grid, logging::error, not_found},
        parameter_checks,
    },
    config::Config,
    installer::{
        Installer, InstallerOptions,
        types::{OptionalPackageId, PackageName, Version},
    },
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Updates the specified package to the new version, or the latest version if no new version is specified.
/// If multiple packages are specified they are all updated to the latest version.
/// If multiple versions of the same package are installed, the <VERSION> option is required, note that this
/// cannot be done with multiple packages at once.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// The packages to update specified with <PACKAGE-NAME>[@<VERSION>] ...
    #[arg(conflicts_with = "all")]
    packages: Vec<OptionalPackageId>,

    /// The version to update to. This can only be a higher version than the current version and can only be used when a single package is specified
    #[arg(long, requires = "packages")]
    new_version: Option<Version>,

    /// Updates all the installed packages to the latest version possible
    #[arg(long, default_value = "false", conflicts_with = "packages")]
    all: bool,

    /// Exclude packages when using the `--all` flag, specified with <PACKAGE-NAME> ...
    #[arg(long, requires = "all")]
    exclude: Vec<PackageName>,
}

impl HandleCommand for UpdateArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let options = InstallerOptions::default();
        let installer = Installer::new(&config, &mut register, &manager, options);

        // If `--all` is specified use all the updatable pacakges
        let optional_ids = match self.all {
            true => &self.get_updatables(installer),
            false if self.packages.is_empty() => {
                error!(msg: "No packages specified to update");
                exit(1);
            },
            false => &self.packages,
        };

        if optional_ids.len() > 1 && self.new_version.is_some() {
            error!(msg: "Cannot specify new version when multiple packages are given");
            exit(1);
        }

        // Check for duplicates, because updating twice will result in an error
        let duplicates = parameter_checks::get_duplicates(optional_ids);
        if !duplicates.is_empty() {
            let mut duplicate_string = String::new();
            for duplicate in duplicates {
                duplicate_string.push_str(&duplicate.to_string());
                duplicate_string.push(' ');
            }

            error!(msg: "Duplicate package arguments are not allowed. The following duplicates were found: {duplicate_string}");
            exit(1);
        }

        for optional_id in optional_ids {
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

            match new_package_id {
                Some(new_package_id) => {
                    println!("Successfully updated {} to {new_package_id}", optional_id);

                    // Save changes
                    register.save_to(&register_dir).unwrap_or_exit(1);
                },
                None => println!("{} is up-to-date!", optional_id.name),
            }
        }
    }
}

impl UpdateArgs {
    /// Gets the updatables and prints them. It will also exclude the packages specified with the exclude flag.
    /// Returns the updatables or exits with status 0 in case all packages are up-to-date.
    pub fn get_updatables(&self, installer: Installer) -> Vec<OptionalPackageId> {
        let updatables = installer.get_updatables().unwrap_or_exit(1);

        // Filter the packages to exclude
        let mut filtered_updatables = Vec::new();
        for package_id in updatables {
            if !self.exclude.contains(&package_id.name) {
                filtered_updatables.push(package_id);
            }
        }

        if filtered_updatables.is_empty() {
            println!("All packages are up-to-date!");
            exit(0);
        }

        println!("The following packages will be updated:");
        grid::print_grid(&filtered_updatables);

        // Check if the user wants to proceed with the update of the found packages
        let question = "Do you wish to proceed?";
        if ask_user(question, QuestionResponse::Yes).unwrap_or_exit(1).is_no_or_invalid() {
            println!("Update canceled");
            exit(0);
        }

        filtered_updatables.into_iter().map(|p| OptionalPackageId::from(p)).collect()
    }
}
