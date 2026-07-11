// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;
use reqwest::{IntoUrl, StatusCode};

use crate::{
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::{RepositoryError, Result},
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
        let data = self.request_metadata(format!("{}/repository.toml", self.url))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_index_metadata(&self) -> Result<IndexMeta> {
        let data = self.request_metadata(format!("{}/index.toml", self.url))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        let data = self.request_metadata(format!("{}/packages/{package}/package.toml", self.url))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        let data = self.request_metadata(format!("{}/packages/{package}/{version}/targets.toml", self.url))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_file_bytes(&self, package: &PackageName, file_path: &str) -> Result<Option<Bytes>> {
        let response = requests::get(format!("{}/packages/{package}/{file_path}", self.url))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Return an error if something went wrong with the request (apart from not found error)
        if !response.status().is_success() {
            return Err(RepositoryError::UnsuccessfulRequest(response.status()));
        }

        Ok(Some(response.bytes()?))
    }

    fn read_file(&self, package: &PackageName, file_path: &str) -> Result<Option<String>> {
        let response = requests::get(format!("{}/packages/{package}/{file_path}", self.url))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Return an error if something went wrong with the request (apart from not found error)
        if !response.status().is_success() {
            return Err(RepositoryError::UnsuccessfulRequest(response.status()));
        }

        Ok(Some(response.text()?))
    }
}

impl WebMetadataProvider {
    /// Creates a new web repository provider for the given repository.
    /// Returns `None` if the repository is not of the correct type.
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != WEB_METADATA_PROVIDER_ID {
            return None;
        }

        Some(Self {
            url: repository.url.clone(),
        })
    }

    /// Requests metadata from the given url.
    /// Returns an `Err(RepositoryError::UnsuccessfulRequest)` if the reponse is not a success.
    /// Returns the metadata as String.
    fn request_metadata<T: IntoUrl>(&self, url: T) -> Result<String> {
        let response = requests::get(url)?;

        // Return an error if something went wrong with the request
        if !response.status().is_success() {
            return Err(RepositoryError::UnsuccessfulRequest(response.status()));
        }

        Ok(response.text()?)
    }
}
