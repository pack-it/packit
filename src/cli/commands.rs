mod install;
mod list;
mod repositories;
mod uninstall;

use clap::{Parser, Subcommand};
use thiserror::Error;

use crate::{
    cli::commands::{install::InstallArgs, list::ListArgs, repositories::RepositoryArgs, uninstall::UninstallArgs},
    config::Config,
    installed_packages::InstalledPackagesError,
    installer::error::InstallerError,
    repositories::manager::RepositoryManager,
    verifier::VerifierError,
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
    Repositories(RepositoryArgs),
}

impl Cli {
    pub fn get_instance() -> Self {
        Cli::parse()
    }

    /// Reads and handles the command.
    pub fn handle_command(&self, manager: &RepositoryManager, config: &Config) -> Result<(), CommandError> {
        // Handle commands with user specified arguments
        let args: &dyn HandleCommand = match &self.command {
            Commands::Install(args) => args,
            Commands::Uninstall(args) => args,
            Commands::List(args) => args,
            Commands::Repositories(args) => args,
        };

        args.handle(config, manager)?;

        Ok(())
    }
}

trait HandleCommand {
    fn handle(&self, config: &Config, manager: &RepositoryManager) -> Result<(), CommandError>;
}
