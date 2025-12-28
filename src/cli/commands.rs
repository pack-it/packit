use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use thiserror::Error;

use crate::{
    cli::{self},
    config::Config,
    installed_packages::{InstalledPackageStorage, InstalledPackagesError},
    installer::{error::InstallerError, installer::Installer},
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    verifier::{get_packages, VerifierError},
};

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Error in installer: {0}")]
    InstallerError(#[from] InstallerError),

    #[error("Error while retrieving installed packages info: {0}")]
    InstalledPackagesError(#[from] InstalledPackagesError),

    #[error("Cannot read install directory: {0}")]
    VerifierError(#[from] VerifierError),
}

#[derive(Parser, Debug)]
#[command(name = "Packit", version, about)]
#[command(long_about = "The universal package manager, designed to streamline the experience of installing packages on your system.")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install a package on your system
    Install(InstallArgs),

    /// Uninstall a package from your system
    Uninstall(UninstallArgs),

    /// List all installed packages
    List(ListArgs),

    /// List all configured repositories
    Repositories,

    /// Search a certain package
    Search(SearchArgs),
}

#[derive(Args, Debug)]
struct InstallArgs {
    /// The name of the package to install
    package_name: String,

    /// The version of the package to install
    #[arg(short, long)]
    version: Option<String>,
}

#[derive(Args, Debug)]
struct UninstallArgs {
    /// The name of the package to uninstall
    package_name: String,

    /// The version of the package to uninstall
    #[arg(short, long)]
    version: Option<String>,
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Directory to list all packages of (OPTIONAL)
    directory: Option<PathBuf>,

    /// Flag to indicate a full check (actually check packit install directory)
    #[arg(short, long)]
    use_dir: bool,
}

#[derive(Args, Debug)]
struct SearchArgs {
    /// The name of the package to search
    package_name: String,

    /// The version of the package to search
    #[arg(short, long)]
    version: Option<String>,
}

/// Reads and handles the command.
pub fn handle_command(manager: &RepositoryManager, config: &Config) -> Result<(), CommandError> {
    let installed_dir = InstalledPackageStorage::get_default_path();
    let mut installed_storage = InstalledPackageStorage::from(&installed_dir)?;
    let command = Cli::parse();

    // Handle commands with user specified arguments
    match command.command {
        Commands::Install(args) => {
            if installed_storage.get_package_versions(&args.package_name).len() >= 1 {
                println!("Package '{}' already exists.", args.package_name);
            } else {
                Installer::new(&config, &mut installed_storage).install(manager, &args.package_name, args.version)?
            }
        },
        Commands::Uninstall(args) => Installer::new(&config, &mut installed_storage).uninstall(&args.package_name, args.version)?,
        Commands::List(args) => handle_list(args, &installed_storage, &config)?,
        Commands::Repositories => handle_repositories(config, manager),
        Commands::Search(args) => handle_search(args, manager),
    }

    // Save changes
    installed_storage.save_to(&installed_dir)?; // TODO: shouldn't this only be done when actually changing the file?

    Ok(())
}

/// Handles the list command with user specified arguments.
fn handle_list(args: ListArgs, installed_storage: &InstalledPackageStorage, config: &Config) -> Result<(), VerifierError> {
    if args.use_dir {
        for package in get_packages(&config)? {
            println!("{}", package);
        }
    } else {
        for package in &installed_storage.installed_packages {
            println!("{} {}", package.name, package.version);
        }
    }

    Ok(())
}

/// Handles the repositories command, listing all configured repositories.
fn handle_repositories(config: &Config, manager: &RepositoryManager) {
    let mut first = true;

    for (repository_id, repository) in &config.repositories {
        if !first {
            println!();
        }
        first = false;

        // Read metadata of repository
        let metadata = match manager.read_repository_metadata(&repository_id) {
            Ok(metadata) => metadata,
            Err(e) => {
                // Display the error and continue
                cli::display_warning(&format!("Cannot read repository metadata of repository '{repository_id}'"));
                cli::display_warning(&format!("{e}"));
                continue;
            },
        };

        // Print repository information
        println!("{} ({repository_id})", metadata.name.bold().blue());
        println!("{}", metadata.description.green());
        println!("Maintainers: {}", metadata.maintainers.join(", "));
        println!("Repository provider: {}, path: {}", repository.provider, repository.path);
    }
}

/// Handles the search command, searching a certain package.
fn handle_search(args: SearchArgs, manager: &RepositoryManager) {
    let (repository_id, package) = match manager.read_package(&args.package_name) {
        Ok(package) => package,
        Err(RepositoryError::PackageNotFoundError { .. }) => {
            println!("Cannot find package {}", args.package_name);
            return;
        },
        Err(e) => {
            println!("Cannot read package: {e:?}");
            return;
        },
    };

    // Get latest version of package
    let latest_version = match package.latest_versions.get(TARGET_ARCHITECTURE) {
        Some(version) => version.to_string(),
        None => {
            println!("Package does not exist for current target");
            return;
        },
    };

    // Use the latest version if the version isn't specified
    let version = match args.version {
        Some(version) => version,
        None => latest_version.clone(),
    };

    // Get package version info for its target
    let package_version = match manager.read_repo_package_version(&repository_id, &package.name, &version) {
        Ok(package_version) => package_version,
        Err(_) => {
            println!("Cannot read {} version {version} from repository {repository_id}", package.name);
            return;
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
            return;
        },
    };

    let dependencies: Vec<_> = package_version.dependencies.iter().chain(target.dependencies.iter()).map(|x| x.as_str()).collect();

    // Print package information
    println!("{} ({})", package.name.bold().blue(), package_version.version);
    println!("{}", package.description.green());
    println!("Latest version: {}", latest_version.red());
    println!("Dependencies: {}", dependencies.join(", ").red());
}
