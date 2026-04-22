// SPDX-License-Identifier: GPL-3.0-only
use reqwest::StatusCode;

use crate::{
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::Result,
        provider::MetadataProvider,
        types::{IndexMeta, PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
    utils::requests,
};

pub const WEB_METADATA_PROVIDER_ID: &str = "web";

/// The web packit repository provider, reading package metadata from an external repository.
pub struct WebMetadataProvider {
    url: String,
}

impl MetadataProvider for WebMetadataProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMeta> {
        let data = requests::get(format!("{}/repository.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_index_metadata(&self) -> Result<IndexMeta> {
        let data = requests::get(format!("{}/index.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        let data = requests::get(format!("{}/packages/{package}/package.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        let data = requests::get(format!("{}/packages/{package}/{version}/targets.toml", self.url))?.text()?;

        Ok(toml::de::from_str(&data)?)
    }

    /// Reads a file and returns its content as a string. If the file doesn't exist Ok(None) is returned.
    /// Returns a response error if `requests::get` or `Response::text` fails.
    fn read_file(&self, package: &PackageName, file_path: &str) -> Result<Option<String>> {
        let response = requests::get(format!("{}/packages/{package}/{file_path}", self.url))?;

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
