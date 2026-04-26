// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use regex::Regex;
use std::{collections::HashSet, str::FromStr};

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, print_grid},
    },
    config::Config,
    installer::types::OptionalPackageId,
    platforms::Target,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::{fuzzy, unwrap_or_exit::UnwrapOrExit},
};

/// Searches a package with `<PACKAGE-NAME>` and shows information based on the package metadata.
/// Alternatively, when the regex flag is true, it uses the regex query to search for packages which match the regex.
/// If the version is given that specific version is searched for.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The query to search with (can be an `OptionalPackageId` or regex string)
    query: String,

    #[arg(long, default_value = "false")]
    regex: bool,
}

impl HandleCommand for SearchArgs {
    /// Handles the search command, searching a certain package.
    fn handle(&self) {
        match self.regex {
            true => self.regex_search(),
            false => self.search_package(),
        }
    }
}

impl SearchArgs {
    /// Searches through all repositories (using their index.toml) for packages which match the given regex.
    /// Fails if the given query is not valid regex.
    fn regex_search(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);

        let regex = Regex::new(&self.query).unwrap_or_exit_msg("Invalid regex pattern", 1);
        let mut matches = HashSet::new();
        for repository_id in &config.repositories_rank {
            let index_meta = manager.read_index_metadata(repository_id).unwrap_or_exit(1);
            for package in index_meta.supported_packages {
                if regex.is_match(&package) {
                    matches.insert(package);
                }
            }
        }

        // Return early if no matches are found
        if matches.is_empty() {
            println!("No packages matched the regex");
            return;
        }

        print_grid(matches.into_iter().collect());
    }

    /// Searches information of a package based on the provider `OptionalPackageId`.
    /// Fails if the given query is not a valid `OptionalPackageId`.
    fn search_package(&self) {
        // Get the optional id
        let message = "The given search query isn't a valid `OptionalPackageId`. For regex use `--regex`.";
        let optional_id = OptionalPackageId::from_str(&self.query).unwrap_or_exit_msg(message, 1);

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let (repository_id, package) = match manager.read_package(&optional_id.name) {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { .. }) => {
                println!("Cannot find package '{}'", optional_id.name);

                let fuzzy_match = fuzzy::repository_search(&config, &manager, &optional_id.name).unwrap_or_exit(1);
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
        let package_id = optional_id.versioned_or(latest_version.clone());

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
