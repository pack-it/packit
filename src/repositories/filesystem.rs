use std::{fs, path::PathBuf};

use crate::{
    config::Repository,
    installer::types::Version,
    repositories::{
        error::Result,
        provider::RepositoryProvider,
        types::{Package, PackageVersion, RepositoryMetadata},
    },
};

pub const FILESYSTEM_PROVIDER_ID: &str = "fs";

/// The filesystem repository provider, reading package metadata from a local repository on the filesystem.
pub struct FileSystemProvider {
    path: PathBuf,
}

impl RepositoryProvider for FileSystemProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata> {
        let data = fs::read_to_string(self.path.join("repository.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &str) -> Result<Package> {
        let data = fs::read_to_string(self.path.join("packages").join(package).join("package.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &str, version: &Version) -> Result<PackageVersion> {
        let path = self.path.join("packages").join(package).join(version.to_string()).join("targets.toml");
        let data = fs::read_to_string(path)?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_script(&self, package: &str, script_path: &str) -> Result<Option<String>> {
        let path = self.path.join("packages").join(package).join(script_path);

        if !fs::exists(&path)? {
            return Ok(None);
        }

        Ok(Some(fs::read_to_string(&path)?))
    }
}

impl FileSystemProvider {
    /// Creates a new filesystem repository provider for the given repository.
    /// Returns None if the repository is not of the correct type.
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != FILESYSTEM_PROVIDER_ID {
            return None;
        }

        Some(FileSystemProvider {
            path: PathBuf::from(repository.path.clone()),
        })
    }
}
