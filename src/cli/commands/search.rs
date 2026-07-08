// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use regex::Regex;
use std::{collections::HashSet, str::FromStr};

use crate::{
    cli::{
        commands::HandleCommand,
        display::{self, deprecation, logging::error, not_found, styled::Styled},
    },
    config::Config,
    installer::types::OptionalPackageId,
    platforms::Target,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Searches a package with `<PACKAGE-NAME>` and shows information based on the package metadata.
/// Alternatively, when the regex flag is true, it uses the regex query to search for packages which match the regex.
/// If the version is given that specific version is searched for.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The query to search with (can be an `OptionalPackageId` or regex string)
    query: String,

    /// True if the query should be interpreted as regex
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
    /// Searches through all repositories for packages which match the given regex.
    /// Fails if the given query is not valid regex.
    fn regex_search(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);

        let regex = Regex::new(&self.query).unwrap_or_exit_msg("Invalid regex pattern", 1);
        let mut matches = HashSet::new();
        for repository_id in manager.iter_supported_repositories_rank() {
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

        display::print_grid(&matches.into_iter().map(|p| p.style()).collect());
    }

    /// Searches information of a package based on the provided `OptionalPackageId`.
    /// Fails if the given query is not a valid `OptionalPackageId`.
    fn search_package(&self) {
        // Get the optional id
        let message = "The given search query isn't a valid `OptionalPackageId`. For regex use `--regex`.";
        let optional_id = OptionalPackageId::from_str(&self.query).unwrap_or_exit_msg(message, 1);

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let (repository_id, package, package_version) = match manager.read_package_and_version(&optional_id, &Target::current()) {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { reason, .. }) => {
                not_found::repository_optional_package(&optional_id, &manager, reason)
            },
            Err(e) => {
                error!(e, "Cannot read package");
                return;
            },
        };

        // Get latest version of package
        let latest_version = match manager.read_latest_supported_version(&repository_id, &package, &Target::current()) {
            Ok(version) => version,
            Err(RepositoryError::PackageNotFoundError { reason, .. }) => {
                println!("Package cannot be found: {reason}");
                return;
            },
            Err(e) => {
                error!(e, "Unable to retrieve latest version of package");
                return;
            },
        };

        // Create a package id
        let package_id = optional_id.versioned_or_cloned(&package_version.version);

        let target_bounds = package_version.get_best_target(&Target::current()).unwrap_or_exit(1);

        // Get current target
        let target = match package_version.get_target(&target_bounds) {
            Ok(target) => target,
            Err(e) => {
                error!(e, "Cannot read {} from repository {repository_id}", package_id.style());
                return;
            },
        };

        let dependencies: Vec<_> = package_version.dependencies.iter().chain(target.dependencies.iter()).collect();
        let dependencies: Vec<String> = dependencies.iter().map(|d| d.to_string()).collect();

        // Print package information
        println!("{} ({})", package.name.style(), package_version.version.style()); // TODO
        println!("{}", package.description.green());
        println!("Latest stable version: {}", latest_version.version.style());
        println!("Dependencies: {}", dependencies.join(", ").bold().blue()); // TODO: List with '-'
        println!("License: {}", package_version.license.to_string().red());

        // Also print revisions if there are any
        if !package_version.revisions.is_empty() {
            println!("Revisions:");
            for revision in &package_version.revisions {
                println!("  - {revision}");
            }
        }

        // Check if the package or version is deprecated
        deprecation::show_package_warnings(&package);
        deprecation::show_package_version_warnings(&package_version, &optional_id.name);
    }
}
