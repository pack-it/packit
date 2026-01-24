mod install;
mod list;
mod repositories;
mod search;
mod uninstall;

use clap::{Parser, Subcommand};

use crate::{
    cli::commands::{install::InstallArgs, list::ListArgs, repositories::RepositoryArgs, search::SearchArgs, uninstall::UninstallArgs},
    config::Config,
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

    /// List all configured repositories
    Repositories(RepositoryArgs),

    /// Search a certain package
    Search(SearchArgs),
}

impl Cli {
    pub fn get_instance() -> Self {
        Cli::parse()
    }

    /// Reads and handles the command.
    pub fn handle_command(&self, manager: &RepositoryManager, config: &Config) {
        // Handle commands with user specified arguments
        let args: &dyn HandleCommand = match &self.command {
            Commands::Install(args) => args,
            Commands::Uninstall(args) => args,
            Commands::List(args) => args,
            Commands::Repositories(args) => args,
            Commands::Search(args) => args,
        };

        args.handle(config, manager);
    }
}

trait HandleCommand {
    fn handle(&self, config: &Config, manager: &RepositoryManager);
}
