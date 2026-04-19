// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::Deserialize;
use thiserror::Error;
use toml_edit::DocumentMut;

use crate::{
    cli::display::logging::warning,
    platforms::{DEFAULT_CONFIG_DIR, DEFAULT_PREFIX},
    repositories::metadata::DEFAULT_METADATA_PROVIDER_ID,
    utils::constants::CONFIG_FILENAME,
};

#[derive(Debug)]
pub struct EditableConfig {
    config: Config,
    document: DocumentMut,
}

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

pub type Result<T> = core::result::Result<T, ConfigError>;

impl Config {
    /// Loads the Packit config from a file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened or if the content is invalid.
    pub fn from(file_path: &Path) -> Result<Self> {
        let file_content = fs::read_to_string(file_path)?;
        let mut config: Self = toml::from_str(&file_content)?;

        // Remove undefined repositories from rank list
        let old_rank_count = config.repositories_rank.len();
        config.repositories_rank.retain(|repo| config.repositories.contains_key(repo));

        if config.repositories_rank.len() < old_rank_count {
            warning!("Repositories rank contains undefined repository, ignoring undefined repository...");
        }

        // Remove trailing slashes from repository paths
        for repository in config.repositories.values_mut() {
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

impl EditableConfig {
    /// Loads the Packit config from a file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened or if the content is invalid.
    pub fn from(file_path: &Path) -> Result<Self> {
        let config = Config::from(file_path)?;
        let file_content = fs::read_to_string(file_path)?;
        let document = DocumentMut::from_str(&file_content).unwrap();

        Ok(Self { config, document })
    }

    /// Saves the Packit config to the given path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be written.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        let data = self.document.to_string();

        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write data to file
        fs::write(path, data)?;

        Ok(())
    }

    /// Sets the repository with the given id.
    pub fn set_repository(&mut self, id: &str, repository: Repository) {
        let mut new_value = toml_edit::Table::new();
        new_value["path"] = (&repository.path).into();
        new_value["provider"] = (&repository.provider).into();
        if let Some(prebuilds_url) = &repository.prebuilds_url {
            new_value["prebuilds_url"] = prebuilds_url.into();
        }
        if let Some(prebuilds_provider) = &repository.prebuilds_provider {
            new_value["prebuilds_provider"] = prebuilds_provider.into();
        }
        self.document["repositories"][id] = new_value.into();

        self.config.repositories.insert(id.into(), repository);
    }

    /// Gets the repository with the given id, returns None when the repository cannot be found.
    pub fn get_repository(&self, id: &str) -> Option<&Repository> {
        self.config.repositories.get(id)
    }

    /// Sets the repositories rank.
    pub fn set_repositories_rank(&mut self, repositories_rank: Vec<String>) {
        let mut new_value = toml_edit::Array::new();
        for repo in &repositories_rank {
            new_value.push(repo);
        }
        self.document["repositories_rank"] = new_value.into();

        self.config.repositories_rank = repositories_rank;
    }

    /// Gets the repositories rank.
    pub fn get_repositories_rank(&self) -> &Vec<String> {
        &self.config.repositories_rank
    }

    /// Sets the prefix directory.
    pub fn set_prefix_directory(&mut self, prefix_directory: PathBuf) {
        self.document["prefix_directory"] = prefix_directory.display().to_string().into();
        self.config.prefix_directory = prefix_directory;
    }

    /// Gets the prefix directory.
    pub fn get_prefix_directory(&self) -> &PathBuf {
        &self.config.prefix_directory
    }

    /// Sets the multiuser mode.
    pub fn set_multiuser(&mut self, multiuser: bool) {
        self.document["multiuser"] = multiuser.into();
        self.config.multiuser = multiuser;
    }

    /// Gets the multiuser mode.
    pub fn is_multiuser(&self) -> bool {
        self.config.multiuser
    }
}
