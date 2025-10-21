use std::{collections::HashMap, fs};

use serde::Deserialize;
use thiserror::Error;

use crate::repositories::default::DEFAULT_PROVIDER_ID;

/// Represents the main config file of Packit.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Contains all repositories
    pub repositories: HashMap<String, Repository>,

    /// Contains the ranks of all repositories
    pub repositories_rank: Vec<String>,

    /// False to use the repositories rank to resolve package conflicts, true to ask for confirmation
    #[serde(default = "default_prompt_repo_conflicts")]
    pub prompt_repo_conflicts: bool, //TODO: maybe remove this option? or should it be optional for extra safety?
}

/// Represents a repository, containing connection information.
#[derive(Deserialize, Debug)]
pub struct Repository {
    /// Path to the repository
    pub path: String,

    /// The repository provider, defaults to the packit repository format
    #[serde(default = "default_repository_provider")]
    pub provider: String,
}

fn default_prompt_repo_conflicts() -> bool {
    false
}

fn default_repository_provider() -> String {
    DEFAULT_PROVIDER_ID.into()
}

/// The errors that occur when reading the config file.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Cannot read config file contents: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
}

impl Config {
    /// Loads a Packit config from a file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened or if the content is invalid.
    pub fn from(file: &str) -> Result<Self, ConfigError> {
        let file_content = fs::read_to_string(file)?;
        let mut config: Self = toml::from_str(&file_content)?;

        // Remove undefined repositories from rank list
        let old_rank_count = config.repositories_rank.len();
        config.repositories_rank.retain(|repo| config.repositories.contains_key(repo));

        if config.repositories_rank.len() < old_rank_count {
            println!("WARNING: Repositories rank contains undefined repository, ignoring undefined repository...")
        }

        // Remove trailing slashes from repository paths
        for (_, repository) in &mut config.repositories {
            repository.path = repository.path.trim_end_matches("/").into();
        }

        Ok(config)
    }
}
