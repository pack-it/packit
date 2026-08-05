// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use regex::Regex;
use std::{
    cmp::max,
    collections::{HashSet, VecDeque},
    process::exit,
    str::FromStr,
};

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
    utils::{
        tree::{Node, Tree},
        unwrap_or_exit::UnwrapOrExit,
    },
};

/// Searches a package with `<PACKAGE-NAME>[@<PACKAGE-VERSION]` and shows information based on the package (version) metadata.
/// Alternatively, when the regex flag is true, it uses the regex query to search for packages which match the regex.
/// Package version specific information is shown when the version is given, otherwise package specific information is shown.
/// Note that the version is necessary if `--tree` is specified.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The query to search with (can be an `OptionalPackageId` or regex string)
    query: String,

    /// True if the query should be interpreted as regex
    #[arg(long, default_value = "false")]
    regex: bool,

    /// True to show the tree of a package
    /// Note that the tree assumes the latest versions
    #[arg(long, default_value = "false", conflicts_with = "regex")]
    tree: bool,

    /// True if verbose information should be shown
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// True to use the latest version
    #[arg(long, default_value = "false", conflicts_with = "regex")]
    latest: bool,
}

impl HandleCommand for SearchArgs {
    /// Handles the search command, searching a certain package.
    fn handle(&self) {
        if self.regex {
            self.regex_search();
            return;
        }

        // Get the optional id
        let message = "The given search query isn't a valid package. For regex use `--regex`.";
        let optional_id = OptionalPackageId::from_str(&self.query).unwrap_or_exit_msg(message, 1);

        // Check if there is version ambiguity (version and `--latest` specified)
        if optional_id.version.is_some() && self.latest {
            error!(msg: "Version is ambiguous, version and `--latest` are both specified");
            exit(1);
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);

        // Get the package version
        let package_version = match &optional_id.version {
            Some(version) => Some(version.clone()),
            None if self.latest => {
                let (repository_id, package_meta) = manager.read_package(&optional_id.name).unwrap_or_exit(1);
                let version_meta =
                    manager.read_latest_supported_version(&repository_id, &package_meta, &Target::current()).unwrap_or_exit(1);
                Some(version_meta.version)
            },
            None => None,
        };

        // Version cannot be none if `--tree` is specified
        if self.tree && package_version.is_none() {
            error!(msg: "The given search query isn't a valid package id. Use `--latest` for the latest version.");
            exit(1);
        }

        match package_version {
            Some(version) if self.tree => self.search_tree(&manager, PackageId::new(optional_id.name, version.clone())),
            Some(version) => self.search_package_version(&manager, &PackageId::new(optional_id.name, version.clone())),
            None => self.search_package(&manager, &optional_id.name),
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

    /// Searches the tree of a given package, always using the latest version for the current target of each dependency.
    /// Fails if the given query is not a valid `PackageId`.
    fn search_tree(&self, manager: &RepositoryManager, package_id: PackageId) {
        let root = Node::new(package_id, (), ());
        let mut tree = Tree::new(root);

        let mut package_queue = VecDeque::from([0]);
        while let Some(node_index) = package_queue.pop_front() {
            let node = tree.get_node_by_index_mut(node_index).expect("Expected node to exist");

            let (_, _, version_meta) =
                manager.read_package_and_version(&node.get_package_id().clone().into(), &Target::current()).unwrap_or_exit(1);

            let target_bounds = version_meta.get_best_target(&Target::current()).unwrap_or_exit(1);
            let target = version_meta.get_target(&target_bounds).unwrap_or_exit(1);
            let dependencies = version_meta.dependencies.iter().chain(target.dependencies.iter());
            let build_dependencies = version_meta.build_dependencies.iter().chain(target.build_dependencies.iter());

            for dependency in build_dependencies.chain(dependencies) {
                let (repository_id, dependency_meta) = manager.read_package(dependency.get_name()).unwrap_or_exit(1);
                let dependency_version_meta = manager
                    .read_latest_supported_dependency_version(&repository_id, &dependency_meta, dependency, &Target::current())
                    .unwrap_or_exit(1);

                let dependency_id = PackageId::new(dependency.get_name().clone(), dependency_version_meta.version);
                let new_node = Node::new(dependency_id, (), ());
                let new_index = tree.add_node(node_index, new_node).unwrap_or_exit(1);
                package_queue.push_back(new_index);
            }
        }

        println!("{tree}");
    }

    /// Searches for and shows package specific information for a given package.
    fn search_package(&self, manager: &RepositoryManager, package_name: &PackageName) {
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
    fn search_package_version(&self, manager: &RepositoryManager, package_id: &PackageId) {
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

        let required_packit_version = max(&package.required_packit_version, &package_version.required_packit_version);

        // Show package version information
        println!("{}", package_id.style());
        println!("{}", package.description.italic().cyan());
        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Homepage", package.homepage.display());
        pair_aligner.add("License", &package_version.license);
        pair_aligner.add("Required Packit version", required_packit_version.display_or(|v| v.style()));
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
