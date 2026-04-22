// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::OptionalPackageId,
    platforms::Target,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::{fuzzy, unwrap_or_exit::UnwrapOrExit},
};

/// Searches a package with <PACKAGE-NAME> and shows information based on the package metadata.
/// If the version is given that specific version is searched for.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The name of the package to install, with an optional version specified with NAME@VERSION
    optional_id: OptionalPackageId,
}

impl HandleCommand for SearchArgs {
    /// Handles the search command, searching a certain package.
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let (repository_id, package) = match manager.read_package(&self.optional_id.name) {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { .. }) => {
                println!("Cannot find package '{}'", self.optional_id.name);

                let fuzzy_match = fuzzy::repository_search(&config, &manager, &self.optional_id.name).unwrap_or_exit(1);
                if let Some(fuzzy_match) = fuzzy_match {
                    println!("Did you mean: '{fuzzy_match}'?");
                }

                return;
            },
            Err(e) => {
                error!(e, "Cannot read package");
                return;
            },
        };

        // Get latest version of package
        let latest_version = match package.get_latest_version(&Target::current()) {
            Ok(version) => version,
            Err(RepositoryError::TargetError) => {
                println!("Package does not exist for current target");
                return;
            },
            Err(e) => {
                error!(e, "Unable to retrieve latest version of package");
                return;
            },
        };

        // Create a package id
        let package_id = self.optional_id.versioned_or(latest_version.clone());

        // Get package version info for its target
        let package_version = match manager.read_repo_package_version(&repository_id, &package_id) {
            Ok(package_version) => package_version,
            Err(e) => {
                error!(e, "Cannot read '{package_id}' from repository {repository_id}");
                return;
            },
        };

        let target_bounds = package_version
            .get_best_target(&Target::current())
            .unwrap_or_exit_msg("The package is not available for the current target", 1);

        // Get current target
        let target = match package_version.get_target(&target_bounds) {
            Ok(target) => target,
            Err(RepositoryError::TargetError) => {
                println!("Package {package_id} from repository {repository_id} does not exist for current target");
                return;
            },
            Err(e) => {
                error!(e, "Cannot read {package_id} from repository {repository_id}");
                return;
            },
        };

        let dependencies: Vec<_> = package_version.dependencies.iter().chain(target.dependencies.iter()).collect();
        let dependencies: Vec<String> = dependencies.iter().map(|d| d.to_string()).collect();

        // Print package information
        println!("{} ({})", package.name.bold().blue(), package_version.version);
        println!("{}", package.description.green());
        println!("Latest version: {}", latest_version.to_string().red());
        println!("Dependencies: {}", dependencies.join(", ").red());
        println!("License: {}", package_version.license.to_string().red());

        // Also print revisions if there are any
        if !package_version.revisions.is_empty() {
            println!("Revisions:");
            for revision in package_version.revisions {
                println!("  - {revision}");
            }
        }
    }
}
