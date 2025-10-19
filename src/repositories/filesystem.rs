use std::fs;

use crate::{config::Repository, repositories::{error::Result, provider::RepositoryProvider, types::{Package, PackageMetadata, PackageVersion, RepositoryMetadata}}};

pub struct FileSystemProvider {
    path: String,
}

impl RepositoryProvider for FileSystemProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata> {
        let data = fs::read_to_string(format!("{}/repository.toml", self.path))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: String) -> Result<Package> {
        let data = fs::read_to_string(format!("{}/packages/{package}/package.toml", self.path))?;

        let content: PackageMetadata = toml::de::from_str(&data)?;
        Ok(content.package)
    }

    fn read_package_version(&self, package: String, version: String) -> Result<PackageVersion> {
        let data = fs::read_to_string(format!("{}/packages/{package}/{version}/targets.toml", self.path))?;

        Ok(toml::de::from_str(&data)?)
    }
}

impl FileSystemProvider {
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != "fs" {
            return None;
        }

        Some(FileSystemProvider {
            path: repository.path.clone(),
        })
    }
}
