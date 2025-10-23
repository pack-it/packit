use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::{
    installer::{error::InstallerError, installer::Installer},
    repositories::manager::RepositoryManager,
};

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
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Directory to list all packages of (OPTIONAL)
    directory: Option<PathBuf>,
}

/// Reads and handles the command.
pub fn handle_command(manager: &RepositoryManager) -> Result<(), InstallerError> {
    let command = Cli::parse();

    match command.command {
        Commands::Install(args) => {
            handle_install(args, manager)?;
        },
        Commands::Uninstall(args) => {
            handle_uninstall(args)?;
        },
        Commands::List(args) => {
            handle_list(args)?;
        },
    }
    Ok(())
}

/// Handles the install command with user specified arguments.
fn handle_install(args: InstallArgs, manager: &RepositoryManager) -> Result<(), InstallerError> {
    // TODO: Get an install directory from the config
    let installer = Installer::new("./temp".into());
    installer.install(manager, &args.package_name, args.version)?;
    Ok(())
}

/// Handles the uninstall command with user specified arguments.
fn handle_uninstall(args: UninstallArgs) -> Result<(), InstallerError> {
    todo!()
}

/// Handles the list command with user specified arguments.
fn handle_list(args: ListArgs) -> Result<(), InstallerError> {
    todo!()
}
