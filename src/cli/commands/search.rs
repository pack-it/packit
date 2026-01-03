use clap::Args;
use colored::Colorize;

use crate::{
    cli::commands::{CommandError, HandleCommand},
    config::Config,
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    version::Version,
};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The name of the package to search
    package_name: String,

    /// The version of the package to search
    #[arg(short, long)]
    version: Option<Version>,
}

impl HandleCommand for SearchArgs {
    /// Handles the search command, searching a certain package.
    fn handle(&self, _: &Config, manager: &RepositoryManager) -> Result<(), CommandError> {
        let (repository_id, package) = match manager.read_package(&self.package_name) {
            Ok(package) => package,
            Err(RepositoryError::PackageNotFoundError { .. }) => {
                println!("Cannot find package {}", self.package_name);
                return Ok(());
            },
            Err(e) => {
                println!("Cannot read package: {e:?}");
                return Ok(());
            },
        };

        // Get latest version of package
        let latest_version = match package.latest_versions.get(TARGET_ARCHITECTURE) {
            Some(version) => version,
            None => {
                println!("Package does not exist for current target");
                return Ok(());
            },
        };

        // Use the latest version if the version isn't specified
        let version = match &self.version {
            Some(version) => version,
            None => &latest_version,
        };

        // Get package version info for its target
        let package_version = match manager.read_repo_package_version(&repository_id, &package.name, &version) {
            Ok(package_version) => package_version,
            Err(e) => {
                println!("Cannot read {} version {version} from repository {repository_id}", package.name);
                println!("{e}");
                return Ok(());
            },
        };

        // Get current target
        let target = match package_version.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => {
                println!(
                    "Package {} version {version} from repository {repository_id} does not exist for current target",
                    package.name
                );
                return Ok(());
            },
        };

        let dependencies: Vec<_> = package_version.dependencies.iter().chain(target.dependencies.iter()).collect();
        let dependencies: Vec<String> = dependencies.iter().map(|d| d.to_string()).collect();

        // Print package information
        println!("{} ({})", package.name.bold().blue(), package_version.version);
        println!("{}", package.description.green());
        println!("Latest version: {}", latest_version.to_string().red());
        println!("Dependencies: {}", dependencies.join(", ").red());

        Ok(())
    }
}
