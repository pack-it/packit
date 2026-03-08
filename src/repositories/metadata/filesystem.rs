use std::{fs, path::PathBuf};

use crate::{
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::Result,
        provider::MetadataProvider,
        types::{PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
};

pub const FILESYSTEM_METADATA_PROVIDER_ID: &str = "fs";

/// The filesystem repository provider, reading package metadata from a local repository on the filesystem.
pub struct FileSystemMetadataProvider {
    path: PathBuf,
}

impl MetadataProvider for FileSystemMetadataProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMeta> {
        let data = fs::read_to_string(self.path.join("repository.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        let data = fs::read_to_string(self.path.join("packages").join(package.to_string()).join("package.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        let path = self.path.join("packages").join(package.to_string()).join(version.to_string()).join("targets.toml");
        let data = fs::read_to_string(path)?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_script(&self, package: &PackageName, script_path: &str) -> Result<Option<String>> {
        let path = self.path.join("packages").join(package.to_string()).join(script_path);

        if !fs::exists(&path)? {
            return Ok(None);
        }

        Ok(Some(fs::read_to_string(&path)?))
    }
}

impl FileSystemMetadataProvider {
    /// Creates a new filesystem repository provider for the given repository.
    /// Returns None if the repository is not of the correct type.
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != FILESYSTEM_METADATA_PROVIDER_ID {
            return None;
        }

        Some(Self {
            path: PathBuf::from(repository.path.clone()),
        })
    }
}
