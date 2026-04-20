// SPDX-License-Identifier: GPL-3.0-only
use clap::Subcommand;
use colored::Colorize;

use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::{Config, EditableConfig, Repository},
    repositories::{manager::RepositoryManager, metadata::DEFAULT_METADATA_PROVIDER_ID},
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

    /// Adds a new repository to the config
    Add {
        /// The id of the new repository
        id: String,

        /// The url of the new repository
        url: String,

        /// The optional provider of the new repository, `web` is used as default
        provider: Option<String>,
    },
}

impl HandleCommand for ConfigArgs {
    /// Handles the config command
    fn handle(&self) {
        let config = EditableConfig::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);

        match self {
            ConfigArgs::Show => self.handle_show(config),
            ConfigArgs::Repositories(RepositoriesArgs::List) => self.handle_list_repositories(config),
            ConfigArgs::Repositories(RepositoriesArgs::Add { id, url, provider }) => self.handle_add_repository(config, id, url, provider),
        }
    }
}

impl ConfigArgs {
    /// Handles the config show command.
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

    /// Handles the config repositories list command
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

    /// Handles the config repositories add command
    fn handle_add_repository(&self, mut config: EditableConfig, id: &str, url: &str, provider: &Option<String>) {
        let provider = match provider {
            Some(provider) => provider,
            None => DEFAULT_METADATA_PROVIDER_ID,
        };
        let repository = Repository::new(url, provider);

        config.set_repository(id, repository);

        config.add_to_repositories_rank(id);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        println!("Succesfully added repository {id} to the config!");
    }
}
