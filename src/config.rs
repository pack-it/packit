use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    platforms::{DEFAULT_CONFIG_DIR, DEFAULT_PREFIX},
    repositories::metadata::DEFAULT_METADATA_PROVIDER_ID,
    utils::constants::CONFIG_FILENAME,
};

/// Represents the main config file of Packit.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Contains all repositories
    pub repositories: HashMap<String, Repository>,

    /// Contains the ranks of all repositories
    pub repositories_rank: Vec<String>,

    /// The prefix directory to use
    #[serde(default = "Config::default_prefix_directory")]
    pub prefix_directory: PathBuf,

    /// True to enable multiuser mode, false to use singleuser mode
    #[serde(default = "Config::default_multiuser")]
    pub multiuser: bool,
}

/// Represents a repository, containing connection information.
#[derive(Deserialize, Debug)]
pub struct Repository {
    /// The path to the repository
    pub path: String,

    /// The repository provider, defaults to the packit repository format
    #[serde(default = "Repository::default_repository_provider")]
    pub provider: String,

    /// The url of the prebuild packages repository
    pub prebuilds_url: Option<String>,

    /// The provider of the prebuild packages repository
    pub prebuilds_provider: Option<String>,
}

impl Repository {
    /// Creates a new repository with the specified path and provider
    pub fn new(path: &str, provider: &str) -> Self {
        Self {
            path: path.to_string(),
            provider: provider.to_string(),
            prebuilds_url: None,
            prebuilds_provider: None,
        }
    }

    /// Specifies the default repository provider id
    pub fn default_repository_provider() -> String {
        DEFAULT_METADATA_PROVIDER_ID.into()
    }
}

/// The errors that occur when reading the config file.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Cannot read config file contents")]
    ReadError(#[from] std::io::Error),

    #[error("Cannot parse config file")]
    ParseError(#[from] toml::de::Error),
}

impl Config {
    /// Loads a Packit config from a file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened or if the content is invalid.
    pub fn from(file_path: &Path) -> Result<Self, ConfigError> {
        let file_content = fs::read_to_string(file_path)?;
        let mut config: Self = toml::from_str(&file_content)?;

        // Remove undefined repositories from rank list
        let old_rank_count = config.repositories_rank.len();
        config.repositories_rank.retain(|repo| config.repositories.contains_key(repo));

        if config.repositories_rank.len() < old_rank_count {
            warning!("Repositories rank contains undefined repository, ignoring undefined repository...");
        }

        // Remove trailing slashes from repository paths
        for (_, repository) in &mut config.repositories {
            repository.path = repository.path.trim_end_matches("/").into();
        }

        Ok(config)
    }

    /// Gets the default path of the Packit config file.
    pub fn get_default_path() -> PathBuf {
        Path::new(DEFAULT_CONFIG_DIR).join(CONFIG_FILENAME)
    }

    fn default_prefix_directory() -> PathBuf {
        DEFAULT_PREFIX.into()
    }

    fn default_multiuser() -> bool {
        false
    }
}
