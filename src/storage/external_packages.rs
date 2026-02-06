use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{installer::types::Version, storage::error::InstalledPackagesError};

/// Represents a package which is installed on the system.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalRegister {
    pub name: String,
    pub version: Option<Version>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub install_path: PathBuf,
}

impl ExternalRegister {
    /// Loads the installed packages file from the given path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened or if the content is invalid.
    pub fn from(path: &Path) -> Result<Self, InstalledPackagesError> {
        // If the file does not exist, we return an empty storage
        if !fs::exists(path)? {
            return Ok(InstalledPackageStorage {
                installed_packages: Vec::new(),
            });
        }

        // Read data from file
        let file_content = fs::read_to_string(path)?;

        // If the file is empty, return an empty storage
        if file_content.trim().is_empty() {
            return Ok(InstalledPackageStorage {
                installed_packages: Vec::new(),
            });
        }

        // Parse data and return
        Ok(toml::from_str(&file_content)?)
    }
}
