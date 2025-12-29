use clap::{Parser, Subcommand};

use crate::{
    cli::{
        error::CommandError,
        subcommands::{install::InstallArgs, list::ListArgs, repositories::handle_repositories, uninstall::UninstallArgs},
    },
    config::Config,
    installed_packages::InstalledPackageStorage,
    repositories::manager::RepositoryManager,
    utils::constants::INSTALLED_DIR,
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
    Repositories,
}

impl Cli {
    pub fn get_instance() -> Self {
        Cli::parse()
    }

    /// Reads and handles the command.
    pub fn handle_command(&self, manager: &RepositoryManager, config: &Config) -> Result<(), CommandError> {
        let installed_dir = config.install_directory.to_string() + INSTALLED_DIR;
        let mut installed_storage = InstalledPackageStorage::from(&installed_dir)?;

        // Handle commands with user specified arguments
        match &self.command {
            Commands::Install(args) => args.handle(config, &mut installed_storage, manager)?,
            Commands::Uninstall(args) => args.handle(config, &mut installed_storage, manager)?,
            Commands::List(args) => args.handle(&installed_storage, &config)?,
            Commands::Repositories => handle_repositories(config, manager),
        }

        // Save changes
        installed_storage.save_to(&installed_dir)?;

        Ok(())
    }
}
