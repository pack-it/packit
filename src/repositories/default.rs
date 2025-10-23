use crate::{
    config::Repository,
    repositories::{
        error::Result,
        provider::RepositoryProvider,
        types::{Package, PackageVersion, RepositoryMetadata},
    },
};

pub const DEFAULT_PROVIDER_ID: &str = "packit";

/// The default packit repository provider, reading package metadata from an external repository.
pub struct DefaultProvider {
    url: String,
}

impl RepositoryProvider for DefaultProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata> {
        let data = reqwest::blocking::get(format!("{}/repository.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &str) -> Result<Package> {
        let data = reqwest::blocking::get(format!("{}/packages/{package}/package.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &str, version: &str) -> Result<PackageVersion> {
        let data = reqwest::blocking::get(format!("{}/packages/{package}/{version}/targets.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }
}

impl DefaultProvider {
    /// Creates a new default repository provider for the given repository.
    /// Returns None if the repository is not of the correct type.
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != DEFAULT_PROVIDER_ID {
            return None;
        }

        Some(DefaultProvider {
            url: repository.path.clone(),
        })
    }
}
