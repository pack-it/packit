use std::fs;

use crate::{
    config::Repository,
    repositories::{
        error::Result,
        provider::RepositoryProvider,
        types::{Package, PackageVersion, RepositoryMetadata},
    },
};

pub const FILESYSTEM_PROVIDER_ID: &str = "fs";

/// The filesystem repository provider, reading package metadata from a local repository on the filesystem.
pub struct FileSystemProvider {
    path: String,
}

impl RepositoryProvider for FileSystemProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata> {
        let data = fs::read_to_string(format!("{}/repository.toml", self.path))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &str) -> Result<Package> {
        let data = fs::read_to_string(format!("{}/packages/{package}/package.toml", self.path))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &str, version: &str) -> Result<PackageVersion> {
        let data = fs::read_to_string(format!("{}/packages/{package}/{version}/targets.toml", self.path))?;

        Ok(toml::de::from_str(&data)?)
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
            path: repository.path.clone(),
        })
    }
}
