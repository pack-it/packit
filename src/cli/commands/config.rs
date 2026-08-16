// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, path::PathBuf, process::exit};

use clap::{ArgAction, Subcommand};
use colored::Colorize;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{
            QuestionResponse,
            aligned_print::PairAligner,
            ask_user,
            logging::{error, warning},
            standard_print::DisplayOption,
            styled::Styled,
        },
    },
    config::{Config, EditableConfig, Repository},
    register::package_register::PackageRegister,
    repositories::{manager::RepositoryManager, metadata::DEFAULT_METADATA_PROVIDER_ID, provider, types::RepositoryMeta},
    utils::{packit_version::current_packit_version, unwrap_or_exit::UnwrapOrExit},
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

        /// True if the repository availability checks should be skipped
        #[arg(long, default_value = "false")]
        unchecked: bool,
    },

    /// Removes a repository from the config
    Remove {
        /// The id of the repository to remove
        id: String,
    },

    /// Sets the repository url and provider
    SetUrl {
        /// The id of the repository to set the url of
        id: String,

        /// The new url of the repository
        url: String,

        /// The new provider of the repository, if no value is given, the old value is used
        provider: Option<String>,

        /// True if the repository availability checks should be skipped
        #[arg(long, default_value = "false")]
        unchecked: bool,
    },

    /// Sets the prebuilds repository url and provider
    SetPrebuilds {
        /// The id of the repository to set the prebuilds url of
        id: String,

        /// The new url of the prebuilds repository
        prebuilds_url: String,

        /// The new provider of the prebuilds repository, if no value is given, the old value is used
        prebuilds_provider: Option<String>,
    },

    /// Disables or enables the prebuilds of a repository
    DisablePrebuilds {
        /// The id of the repository to enable or disable the prebuilds of
        id: String,

        /// True to disable prebuilds, false to enable
        #[arg(action = ArgAction::Set)]
        value: bool,

        /// True if the prebuilds url should be removed
        #[arg(long, default_value = "false")]
        remove_urls: bool,
    },
}

impl HandleCommand for ConfigArgs {
    fn handle(&self) {
        let config = EditableConfig::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);

        match self {
            ConfigArgs::Show => self.handle_show(config),
            ConfigArgs::SetPrefix { new_prefix } => self.handle_set_prefix(config, new_prefix),
            ConfigArgs::SetMultiuser { multiuser } => self.handle_set_multiuser(config, *multiuser),
            ConfigArgs::Repositories(RepositoriesArgs::List) => self.handle_list_repositories(config),
            ConfigArgs::Repositories(RepositoriesArgs::SetRank { new_rank }) => self.handle_set_repositories_rank(config, new_rank),
            ConfigArgs::Repositories(RepositoriesArgs::Add {
                id,
                url,
                provider,
                unchecked,
            }) => self.handle_add_repository(config, id, url, provider, *unchecked),
            ConfigArgs::Repositories(RepositoriesArgs::Remove { id }) => self.handle_remove_repository(config, id),
            ConfigArgs::Repositories(RepositoriesArgs::SetUrl {
                id,
                url,
                provider,
                unchecked,
            }) => self.handle_set_url(config, id, url, provider, *unchecked),
            ConfigArgs::Repositories(RepositoriesArgs::SetPrebuilds {
                id,
                prebuilds_url,
                prebuilds_provider,
            }) => self.handle_set_prebuilds(config, id, prebuilds_url, prebuilds_provider),
            ConfigArgs::Repositories(RepositoriesArgs::DisablePrebuilds { id, value, remove_urls }) => {
                self.handle_disable_prebuilds(config, id, *value, *remove_urls)
            },
        }
    }
}

impl ConfigArgs {
    /// Handles the config show command.
    fn handle_show(&self, config: EditableConfig) {
        config.get_config().display();
    }

    /// Handles the config set-prefix command.
    fn handle_set_prefix(&self, mut config: EditableConfig, new_prefix: &PathBuf) {
        if config.get_config().prefix_directory == *new_prefix {
            println!("The prefix directory is already set to this path!");
            return;
        }

        let register_dir = PackageRegister::get_path(&config.get_config().prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit_msg("Cannot read package register", 1);

        // Check if there are installed packages
        if register.iterate_all().count() > 0 {
            println!("There are currently installed packages, changing the prefix when packages are installed is currently not supported!");
            return;
        }

        config.set_prefix_directory(new_prefix.clone());

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let styled_message = format!("Succesfully changed the prefix directory to '{}'!", new_prefix.display()).bold().green();
        println!("{styled_message}");
    }

    /// Handles the config set-multiuser command.
    fn handle_set_multiuser(&self, mut config: EditableConfig, multiuser: bool) {
        if config.get_config().multiuser == multiuser {
            println!("The multiuser setting is already set to this state!");
            return;
        }

        let register_dir = PackageRegister::get_path(&config.get_config().prefix_directory);
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

        let multiuser_state = match multiuser {
            true => "on!",
            false => "off!",
        };

        let styled_message = format!("Succesfully changed the multiuser setting to {}", multiuser_state).bold().green();
        println!("{styled_message}")
    }

    /// Handles the config repositories list command.
    fn handle_list_repositories(&self, config: EditableConfig) {
        let config = config.get_config();
        let manager = RepositoryManager::new(config);
        for (index, (repository_id, repository)) in config.repositories.iter().enumerate() {
            // Print a newline between the different repositories
            if index != 0 {
                println!();
            }

            // Check if the repository is unsupported
            if let Some(metadata) = manager.get_unsupported_repositories().get(repository_id) {
                println!("{} ({repository_id}) {}", metadata.name.bold().blue(), "NOT SUPPORTED".bold().red());
                println!("{}", metadata.description.italic().cyan());
                let mut pair_aligner = PairAligner::new();
                pair_aligner.add("License", &metadata.license);
                pair_aligner.add("Maintainers", metadata.maintainers.join(", "));
                pair_aligner.add("Repository provider", &repository.provider);
                pair_aligner.add("Repository url", &repository.url);
                pair_aligner.add("Required Packit Version", metadata.required_packit_version.style().red());
                pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
                continue;
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
            println!("{}", metadata.description.italic().cyan());
            let mut pair_aligner = PairAligner::new();
            pair_aligner.add("License", metadata.license);
            pair_aligner.add("Maintainers", metadata.maintainers.join(", "));
            pair_aligner.add("Repository provider", &repository.provider);
            pair_aligner.add("Repository url", &repository.url);
            pair_aligner.add("Required Packit Version", metadata.required_packit_version.style());
            pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        }
    }

    /// Handles the config repositories set-rank command.
    fn handle_set_repositories_rank(&self, mut config: EditableConfig, new_rank: &Vec<String>) {
        let mut seen = HashSet::new();
        for repo in new_rank {
            // Check for invalid repository ids
            if !config.get_config().repositories.contains_key(repo) {
                error!(msg: "Repository '{repo}' does not exist. Please add it to the config first.");
                exit(1);
            }

            // Check for duplicates
            if !seen.insert(repo) {
                error!(msg: "The given repositories rank contains duplicates. Please remove duplicate repository ids.");
                exit(1);
            }
        }

        config.set_repositories_rank(new_rank.clone());

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let styled_message = format!("Succesfully set the repository rank to '{}'!", new_rank.join(", ")).bold().green();
        println!("{styled_message}");
    }

    /// Handles the config repositories add command.
    fn handle_add_repository(&self, mut config: EditableConfig, id: &str, url: &str, provider: &Option<String>, unchecked: bool) {
        // Check if the config already contains a repository with this id
        if config.get_config().repositories.contains_key(id) {
            error!(msg: "A repository with id '{id}' already exists.");
            exit(1);
        }

        let provider = match provider {
            Some(provider) => provider,
            None => DEFAULT_METADATA_PROVIDER_ID,
        };
        let mut repository = Repository::new(url, provider);

        // Only run checks if unchecked is not enabled
        if !unchecked {
            let repo_meta = self.check_metadata_repository_availability(&repository);
            let is_repository_reachable = repo_meta.as_ref().map(|x| self.check_metadata_repository_compatibility(x)).unwrap_or(false);

            // Check if the repository is reachable
            if is_repository_reachable {
                if ask_user("Are you sure you want to add this repository?", QuestionResponse::No).unwrap_or_exit(1).is_no_or_invalid() {
                    println!("Cancelling adding of repository.");
                    return;
                }
            }

            if let Some(repo_meta) = repo_meta {
                // Check if repository suggests a prebuild url
                if let Some(prebuilds_url) = &repo_meta.prebuilds_url {
                    println!("This repository suggests using the following prebuilds repository:");
                    let mut pair_aligner = PairAligner::new();
                    pair_aligner.add("Url", prebuilds_url);
                    pair_aligner.add("Provider", repo_meta.prebuilds_provider.display());
                    pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);

                    if ask_user("Do you want to add this prebuild repository?", QuestionResponse::Yes).unwrap_or_exit(1).is_yes() {
                        println!("Adding prebuild repository to config.");
                        repository.prebuilds_url = Some(prebuilds_url.clone());
                        repository.prebuilds_provider = repo_meta.prebuilds_provider;
                    }
                }
            }
        }

        config.set_repository(id, repository);

        config.add_to_repositories_rank(id);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let styled_message = format!("Succesfully added repository '{id}' to the config!").bold().green();
        println!("{styled_message}");
    }

    /// Handles the config repositories remove command.
    fn handle_remove_repository(&self, mut config: EditableConfig, id: &str) {
        // Check if the config even contains this repository
        if !config.get_config().repositories.contains_key(id) {
            error!(msg: "Repository '{id}' does not exist.");
            exit(1);
        }

        // Check if the config only contains this repository
        if config.get_config().repositories.len() == 1 {
            error!(msg: "Repository '{id}' is the only repository in the config, please add another one before removing this one.");
            exit(1);
        }

        config.remove_repository(id);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let styled_message = format!("Succesfully removed repository '{id}' from the config!").bold().green();
        println!("{styled_message}");
    }

    /// Handles the config repositories set-url command.
    fn handle_set_url(&self, mut config: EditableConfig, id: &str, url: &str, provider: &Option<String>, unchecked: bool) {
        // Check if the config even contains this repository
        let Some(mut repository) = config.get_config().repositories.get(id).cloned() else {
            error!(msg: "Repository '{id}' does not exist.");
            exit(1);
        };

        // Only run checks if unchecked is not enabled
        if !unchecked {
            let repository = Repository::new(url, provider.as_ref().unwrap_or(&repository.provider));
            let repo_meta = self.check_metadata_repository_availability(&repository);
            let is_repository_reachable = repo_meta.map(|x| self.check_metadata_repository_compatibility(&x)).unwrap_or(false);

            // Check if the repository is reachable
            if !is_repository_reachable {
                if ask_user("Are you sure you want to change the url to this repository?", QuestionResponse::No)
                    .unwrap_or_exit(1)
                    .is_no_or_invalid()
                {
                    println!("Cancelling repository url change.");
                    return;
                }
            }
        }

        println!("Overwriting url: {}, provider: {}", repository.url, repository.provider);

        repository.url = url.into();
        if let Some(provider) = provider {
            repository.provider = provider.clone();
        }

        config.set_repository(id, repository);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let styled_message = format!("Succesfully set repository url for '{id}' to '{url}'!").bold().green();
        println!("{styled_message}");
    }

    /// Handles the config repositories set-prebuilds command.
    fn handle_set_prebuilds(&self, mut config: EditableConfig, id: &str, prebuilds_url: &str, prebuilds_provider: &Option<String>) {
        // Check if the config even contains this repository
        let Some(mut repository) = config.get_config().repositories.get(id).cloned() else {
            error!(msg: "Repository '{id}' does not exist.");
            exit(1);
        };

        // Check if a prebuilds repository was already configured
        if let Some(old_prebuilds_url) = &repository.prebuilds_url {
            println!("Repository '{id}' already had a prebuild repository, overwriting url and provider...");
            println!(
                "Old prebuilds url: {old_prebuilds_url}, provider: {}",
                repository.prebuilds_provider.display()
            );
        }

        repository.prebuilds_url = Some(prebuilds_url.into());
        if prebuilds_provider.is_some() {
            repository.prebuilds_provider = prebuilds_provider.clone();
        }

        config.set_repository(id, repository);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let styled_message = format!("Succesfully set prebuilds repository for '{id}' to '{prebuilds_url}'!").bold().green();
        println!("{styled_message}");
    }

    /// Handles the config repositories disable-prebuilds command.
    fn handle_disable_prebuilds(&self, mut config: EditableConfig, id: &str, value: bool, remove_urls: bool) {
        // Check if the config even contains this repository
        let Some(mut repository) = config.get_config().repositories.get(id).cloned() else {
            error!(msg: "Repository '{id}' does not exist.");
            exit(1);
        };

        repository.disable_prebuilds = value;

        // Remove url if prebuilds disabled and `--remove-urls` flag is enabled
        if value && remove_urls {
            repository.prebuilds_url = None;
            repository.prebuilds_provider = None;
        }

        config.set_repository(id, repository);

        config.save_to(&Config::get_default_path()).unwrap_or_exit_msg("Cannot save config file", 1);

        let status = if value { "disabled" } else { "enabled" };
        let styled_message = format!("Succesfully {status} prebuilds for '{id}'!").bold().green();
        println!("{styled_message}");
    }

    /// Check if the metadata repository is available.
    /// Shows a message with the found issues.
    /// Returns the repository metadata if the repository is available, false otherwise.
    fn check_metadata_repository_availability(&self, repository: &Repository) -> Option<RepositoryMeta> {
        // Check if the provider can be created
        let Some(provider) = provider::create_metadata_provider(repository) else {
            warning!("Cannot connect to repository");
            return None;
        };

        // Check if the repository metadata can be read
        let repo_meta = match provider.read_repository_metadata() {
            Ok(repo_meta) => repo_meta,
            Err(e) => {
                warning!("Cannot request repository metadata: {e}");
                return None;
            },
        };

        Some(repo_meta)
    }

    /// Check if the metadata repository is compatible with the current system.
    /// Shows a message with the found issues.
    /// Returns true if the repository is compatible, false otherwise.
    fn check_metadata_repository_compatibility(&self, repo_meta: &RepositoryMeta) -> bool {
        // Check if the repository is supported
        if repo_meta.required_packit_version > current_packit_version() {
            warning!(
                "This repository requires Packit version {}, while your current version is {}",
                repo_meta.required_packit_version.style(),
                current_packit_version().style()
            );

            return false;
        }

        true
    }
}
