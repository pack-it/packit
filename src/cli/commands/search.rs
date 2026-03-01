use clap::Args;
use colored::Colorize;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::OptionalPackageId,
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, manager::RepositoryManager},
};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The name of the package to install, with an optional version specified with NAME@VERSION
    optional_id: OptionalPackageId,
}

impl HandleCommand for SearchArgs {
    /// Handles the search command, searching a certain package.
    fn handle(&self, _: &Config, manager: &RepositoryManager) {
        let (repository_id, package) = match manager.read_package(&self.optional_id.name) {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { .. }) => {
                println!("Cannot find package {}", self.optional_id.name);
                return;
            },
            Err(e) => {
                error!(e, "Cannot read package");
                return;
            },
        };

        // Get latest version of package
        let latest_version = match package.latest_versions.get(TARGET_ARCHITECTURE) {
            Some(version) => version,
            None => {
                println!("Package does not exist for current target");
                return;
            },
        };

        // Use the latest version if the version isn't specified
        let version = match &self.optional_id.version {
            Some(version) => version,
            None => &latest_version,
        };

        // Create a package id
        let package_id = self.optional_id.to_package_id(version.clone());

        // Get package version info for its target
        let package_version = match manager.read_repo_package_version(&repository_id, &package_id) {
            Ok(package_version) => package_version,
            Err(e) => {
                error!(e, "Cannot read {} version {version} from repository {repository_id}", package.name);
                return;
            },
        };

        // Get current target
        let target = match package_version.get_target(TARGET_ARCHITECTURE) {
            Ok(target) => target,
            Err(RepositoryError::TargetError) => {
                println!(
                    "Package {} version {version} from repository {repository_id} does not exist for current target",
                    package.name
                );
                return;
            },
            Err(e) => {
                error!(e, "Cannot read {} version {version} from repository {repository_id}", package.name);
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
        println!("License: {}", package_version.license.unwrap_or("None".to_string()).red());

        // Also print revisions if there are any
        if package_version.revisions.len() > 0 {
            println!("Revisions:");
            for revision in package_version.revisions {
                println!("  - {revision}");
            }
        }
    }
}
