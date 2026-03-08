use reqwest::StatusCode;

use crate::{
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::Result,
        provider::MetadataProvider,
        types::{PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
};

pub const WEB_METADATA_PROVIDER_ID: &str = "web";

/// The web packit repository provider, reading package metadata from an external repository.
pub struct WebMetadataProvider {
    url: String,
}

impl MetadataProvider for WebMetadataProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMeta> {
        let data = reqwest::blocking::get(format!("{}/repository.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        let data = reqwest::blocking::get(format!("{}/packages/{package}/package.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        let data = reqwest::blocking::get(format!("{}/packages/{package}/{version}/targets.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_script(&self, package: &PackageName, script_path: &str) -> Result<Option<String>> {
        let response = reqwest::blocking::get(format!("{}/packages/{package}/{script_path}", self.url))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(response.text()?))
    }
}

impl WebMetadataProvider {
    /// Creates a new web repository provider for the given repository.
    /// Returns None if the repository is not of the correct type.
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != WEB_METADATA_PROVIDER_ID {
            return None;
        }

        Some(Self {
            url: repository.path.clone(),
        })
    }
}
