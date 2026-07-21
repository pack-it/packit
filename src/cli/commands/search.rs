// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use regex::Regex;
use std::{cmp::max, collections::HashSet, str::FromStr};

use crate::{
    cli::{
        commands::HandleCommand,
        display::{
            self,
            aligned_print::PairAligner,
            deprecation,
            logging::error,
            not_found,
            standard_print::{self, DisplayJoined, DisplayOption},
            styled::{MapStyled, Styled},
        },
    },
    config::Config,
    installer::types::{OptionalPackageId, PackageId, PackageName},
    platforms::Target,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Searches a package with `<PACKAGE-NAME>[@<PACKAGE-VERSION]` and shows information based on the package (version) metadata.
/// Alternatively, when the regex flag is true, it uses the regex query to search for packages which match the regex.
/// Package version specific information is shown when the version is given, otherwise package specific information is shown.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The query to search with (can be an `OptionalPackageId` or regex string)
    query: String,

    /// True if the query should be interpreted as regex
    #[arg(long, default_value = "false")]
    regex: bool,

    /// True if verbose information should be shown
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

impl HandleCommand for SearchArgs {
    /// Handles the search command, searching a certain package.
    fn handle(&self) {
        match self.regex {
            true => self.regex_search(),
            false => self.search(),
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

        display::print_grid(&matches.into_iter().map_styled().collect());
    }

    /// Searches information of a package based on the provided `OptionalPackageId`.
    /// Fails if the given query is not a valid `OptionalPackageId`.
    fn search(&self) {
        // Get the optional id
        let message = "The given search query isn't a valid package. For regex use `--regex`.";
        let optional_id = OptionalPackageId::from_str(&self.query).unwrap_or_exit_msg(message, 1);

        match optional_id.versioned() {
            Some(package_id) => self.search_package_version(&package_id),
            None => self.search_package(&optional_id.name),
        }
    }

    /// Searches for and shows package specific information for a given package.
    fn search_package(&self, package_name: &PackageName) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let (repository_id, package) = match manager.read_package(package_name) {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { reason, .. }) => not_found::repository_package(package_name, &manager, reason),
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

        // Print package information
        println!("{}", package_name.style());
        println!("{}", package.description.italic().cyan());
        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Homepage", package.homepage.display());
        pair_aligner.add("Latest version", latest_version.version.style());
        pair_aligner.add("Available versions", package.versions.iter().map_styled().display(" | "));
        pair_aligner.add("Required Packit version", package.required_packit_version.display_or(|v| v.style()));
        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();

        if self.verbose {
            print!("Conflicts with: ");
            standard_print::print_list_or_none(package.conflicts_with.iter());
        }

        // Check if the package is deprecated
        deprecation::show_package_warnings(&package);
    }

    /// Searches for and shows package version specific information for a given package.
    fn search_package_version(&self, package_id: &PackageId) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let package_and_version = manager.read_package_and_version(&package_id.clone().into(), &Target::current());
        let (repository_id, package, package_version) = match package_and_version {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { reason, .. }) => {
                not_found::repository_package_version(package_id, &manager, reason)
            },
            Err(e) => {
                error!(e, "Cannot read package");
                return;
            },
        };

        let target_bounds = package_version.get_best_target(&Target::current()).unwrap_or_exit(1);

        // Get current target
        let target = match package_version.get_target(&target_bounds) {
            Ok(target) => target,
            Err(e) => {
                error!(e, "Cannot read {} from repository '{repository_id}'", package_id.style());
                return;
            },
        };

        // Chain the package version dependencies with the target dependencies
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        let build_dependencies = package_version.build_dependencies.iter().chain(target.build_dependencies.iter());

        let required_packit_version = match max(&package.required_packit_version, &package_version.required_packit_version) {
            Some(version) => version.style(),
            None => "None".dimmed(),
        };

        // Show package version information
        println!("{}", package_id.style());
        println!("{}", package.description.italic().cyan());
        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Homepage", package.homepage.display());
        pair_aligner.add("License", &package_version.license);
        pair_aligner.add("Required Packit version", required_packit_version);
        pair_aligner.add("Skip symlinking", if package_version.skip_symlinking { "on" } else { "off" });
        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();

        print!("Dependencies: ");
        standard_print::print_list_or_none(dependencies);

        if self.verbose {
            print!("Build dependencies: ");
            standard_print::print_list_or_none(build_dependencies);

            print!("Revisions: ");
            standard_print::print_list_or_none(package_version.revisions.iter());
        }

        // Check if the package is deprecated
        deprecation::show_package_version_warnings(&package_version, &package_id.name);
    }
}
