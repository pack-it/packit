// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use clap::{ArgAction, Subcommand};
use colored::Colorize;

use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::{Config, EditableConfig, Repository},
    repositories::{manager::RepositoryManager, metadata::DEFAULT_METADATA_PROVIDER_ID},
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Manages the Packit config file.
#[derive(Subcommand, Debug)]
pub enum ConfigArgs {
    /// Shows the current configuration
    Show,

    /// Changes the prefix directory to the given path
    SetPrefix {
        /// The new prefix to use
        new_prefix: PathBuf,
    },

    /// Changes the multiuser setting to the given new state
    SetMultiuser {
        /// The new multiuser setting
        #[arg(action = ArgAction::Set)]
        multiuser: bool,
    },

    /// Manages the repositories in the config
    #[clap(subcommand)]
    Repositories(RepositoriesArgs),
}

/// Manages the repositories in the config.
#[derive(Subcommand, Debug)]
pub enum RepositoriesArgs {
    /// Lists all configured repositories
    List,

    /// Sets the repositories rank
    SetRank {
        /// The new rank to set
        #[arg(required = true)]
        new_rank: Vec<String>,
    },

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
            ConfigArgs::SetPrefix { new_prefix } => self.handle_set_prefix(config, new_prefix),
            ConfigArgs::SetMultiuser { multiuser } => self.handle_set_multiuser(config, *multiuser),
            ConfigArgs::Repositories(RepositoriesArgs::List) => self.handle_list_repositories(config),
            ConfigArgs::Repositories(RepositoriesArgs::SetRank { new_rank }) => self.handle_set_repositories_rank(config, new_rank),
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

    /// Handles the config set-prefix command
    fn handle_set_prefix(&self, mut config: EditableConfig, new_prefix: &PathBuf) {
        if config.get_config().prefix_directory == *new_prefix {
            println!("The prefix directory is already set to this path!");
            return;
        }

        let register_dir = PackageRegister::get_default_path(&config.get_config());
        let register = PackageRegister::from(&register_dir).unwrap_or_exit_msg("Cannot read package register", 1);

        // Check if there are installed packages
        if register.iterate_all().count() > 0 {
            println!("There are currently installed packages, changing the prefix when packages are installed is currently not supported!");
            return;
        }

        config.set_prefix_directory(new_prefix.clone());

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        println!("Succesfully changed the prefix directory to {}!", new_prefix.display());
    }

    /// Handles the config set-multiuser command
    fn handle_set_multiuser(&self, mut config: EditableConfig, multiuser: bool) {
        if config.get_config().multiuser == multiuser {
            println!("The multiuser setting is already set to this state!");
            return;
        }

        let register_dir = PackageRegister::get_default_path(&config.get_config());
        let register = PackageRegister::from(&register_dir).unwrap_or_exit_msg("Cannot read package register", 1);

        // Check if there are installed packages
        if register.iterate_all().count() > 0 {
            println!(
                "There are currently installed packages, changing the multiuser setting when packages are installed is currently not supported!"
            );
            return;
        }

        config.set_multiuser(multiuser);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        print!("Succesfully changed the multiuser setting to ");
        if multiuser {
            println!("on!");
        } else {
            println!("off!");
        }
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

    /// Handles the config repositories set-rank command
    fn handle_set_repositories_rank(&self, mut config: EditableConfig, new_rank: &Vec<String>) {
        for repo in new_rank {
            if !config.get_config().repositories.contains_key(repo) {
                println!("Repository {repo} does not exist. Please add it to the config first.");
                return;
            }
        }

        config.set_repositories_rank(new_rank.clone());

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        println!("Succesfully set the repository rank to '{}'!", new_rank.join(", "));
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
