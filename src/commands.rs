use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::{
    installer::{error::InstallerError, installer::Installer},
    repositories::provider::RepositoryProvider,
    target_architecture::TARGET_ARCHITECTURE,
};

#[derive(Parser, Debug)]
#[command(name = "Packit", version, about)]
#[command(
    long_about = "The universal package manager, designed to streamline the experience of installing packages on your system."
)]
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

// This executes a packit subcommand
pub fn execute(provider: &impl RepositoryProvider) -> Result<(), InstallerError> {
    let command = Cli::parse();

    match command.command {
        Commands::Install(args) => {
            command_install(args, provider)?;
        }
        Commands::Uninstall(args) => {
            uninstall(args);
        }
        Commands::List(args) => {
            list_(args);
        }
    }
    Ok(())
}

// Executes the install command with user specified arguments
fn command_install(
    args: InstallArgs,
    provider: &impl RepositoryProvider,
) -> Result<(), InstallerError> {
    // If no version is supplied use the latest version
    let version = match args.version {
        Some(version) => version,
        None => provider.read_package(&args.package_name)?.latest_version,
    };

    // TODO: Get an install directory from the config
    let installer = Installer::new("./temp".into());
    installer.install(
        provider,
        &args.package_name,
        Some(&version),
        TARGET_ARCHITECTURE,
    )?;

    Ok(())
}

fn uninstall(args: UninstallArgs) {}

fn list_(args: ListArgs) {}
