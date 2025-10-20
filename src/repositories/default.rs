use crate::{config::Repository, repositories::{error::Result, provider::RepositoryProvider, types::{Package, PackageMetadata, PackageVersion, RepositoryMetadata}}};

pub const DEFAULT_PROVIDER_ID: &str = "packit";

pub struct DefaultProvider {
    url: String,
}

impl RepositoryProvider for DefaultProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata> {
        let data = reqwest::blocking::get(format!("{}/repository.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: String) -> Result<Package> {
        let data = reqwest::blocking::get(format!("{}/packages/{package}/package.toml", self.url))?.text()?;

        let content: PackageMetadata = toml::de::from_str(&data)?;
        Ok(content.package)
    }

    fn read_package_version(&self, package: String, version: String) -> Result<PackageVersion> {
        let data = reqwest::blocking::get(format!("{}/packages/{package}/{version}/targets.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }
}

impl DefaultProvider {
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != DEFAULT_PROVIDER_ID {
            return None;
        }

        Some(DefaultProvider {
            url: repository.path.clone(),
        })
    }
}
