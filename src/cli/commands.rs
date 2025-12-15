use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use thiserror::Error;

use crate::{
    cli::{self},
    config::Config,
    installed_packages::{InstalledPackageStorage, InstalledPackagesError},
    installer::{error::InstallerError, installer::Installer},
    repositories::manager::RepositoryManager,
    utils::constants::INSTALLED_DIR,
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

/// Reads and handles the command.
pub fn handle_command(manager: &RepositoryManager, config: &Config) -> Result<(), CommandError> {
    let installed_dir = config.install_directory.to_string() + INSTALLED_DIR;
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
    }

    // Save changes
    installed_storage.save_to(&installed_dir)?;

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
