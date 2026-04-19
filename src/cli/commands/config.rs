// SPDX-License-Identifier: GPL-3.0-only
use clap::Subcommand;
use colored::Colorize;

use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::{Config, EditableConfig},
    repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Manages the Packit config file.
#[derive(Subcommand, Debug)]
pub enum ConfigArgs {
    /// Shows the current configuration
    Show,

    /// Manages the repositories in the config
    #[clap(subcommand)]
    Repositories(RepositoriesArgs),
}

/// Manages the repositories in the config.
#[derive(Subcommand, Debug)]
pub enum RepositoriesArgs {
    /// Lists all configured repositories
    List,
}

impl HandleCommand for ConfigArgs {
    fn handle(&self) {
        let config = EditableConfig::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);

        match self {
            ConfigArgs::Show => self.handle_show(config),
            ConfigArgs::Repositories(RepositoriesArgs::List) => self.handle_list_repositories(config),
        }
    }
}

impl ConfigArgs {
    fn handle_show(&self, config: EditableConfig) {
        let config = config.get_config();
        println!("Prefix directory: {}", config.prefix_directory.display());

        print!("Multiuser mode: ");
        if config.multiuser {
            println!("on");
        } else {
            println!("off");
        }

        println!("Repositories rank: {}", config.repositories_rank.join(", "));
    }

    fn handle_list_repositories(&self, config: EditableConfig) {
        let config = config.get_config();
        let manager = RepositoryManager::new(config);
        for (index, (repository_id, repository)) in config.repositories.iter().enumerate() {
            if index != 0 {
                println!();
            }

            // Read metadata of repository
            let metadata = match manager.read_repository_metadata(repository_id) {
                Ok(metadata) => metadata,
                Err(e) => {
                    // Display the error and continue
                    warning!("Cannot read repository metadata of repository '{repository_id}'");
                    warning!("{e}");
                    continue;
                },
            };

            // Print repository information
            println!("{} ({repository_id})", metadata.name.bold().blue());
            println!("{}", metadata.description.green());
            println!("License: {}", metadata.license);
            println!("Maintainers: {}", metadata.maintainers.join(", "));
            println!("Repository provider: {}, path: {}", repository.provider, repository.path);
        }
    }
}
