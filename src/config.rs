use std::{collections::HashMap, fs};

use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub repositories: HashMap<String, Repository>,  // Contains all repositories
    pub repositories_rank: Vec<String>,             // Contains the ranks of all repositories
    #[serde(default = "default_prompt_repo_conflicts")]
    pub prompt_repo_conflicts: bool,                // False to use the repositories rank to resolve package conflicts, true to ask for confirmation
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub path: String,       // Path to the repository
    #[serde(default = "default_repository_provider")]
    pub provider: String,   // The repository provider, defaults to the packit repository format
}

fn default_prompt_repo_conflicts() -> bool {
    false
}

fn default_repository_provider() -> String {
    "packit".into()
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Cannot read config file contents")]
    ReadError(#[from] std::io::Error),
    #[error("Cannot parse config file: {message}", message = ._0.message())]
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
        Ok(toml::from_str(&file_content)?)
    }
}
