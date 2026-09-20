// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;
use reqwest::{IntoUrl, StatusCode, blocking::Response};
use url::Url;

use crate::{
    cli::display::logging::debug,
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::{RepositoryError, Result},
        metadata::provider::MetadataProviderImpl,
        types::{IndexMeta, PackageMeta, PackageVersionMeta, PrebuildsList, RepositoryMeta},
    },
    utils::requests,
};

pub const WEB_METADATA_PROVIDER_ID: &str = "web";

/// The web packit repository provider, reading package metadata from an external repository.
pub struct WebMetadataProvider {
    url: String,
}

impl MetadataProviderImpl for WebMetadataProvider {
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

    fn read_prebuilds_list(&self, package: &PackageName, version: &Version) -> Result<Option<PrebuildsList>> {
        let response = requests::get(format!("{}/packages/{package}/{version}/prebuilds.toml", self.url))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Return an error if something went wrong with the request (apart from not found error)
        if !response.status().is_success() {
            return Err(RepositoryError::UnsuccessfulRequest(response.status()));
        }

        Ok(Some(toml::de::from_str(&response.text()?)?))
    }

    fn read_file_bytes(&self, package: &PackageName, file_path: &str) -> Result<Option<Bytes>> {
        let Some(response) = self.request_file(package, file_path)? else {
            return Ok(None);
        };

        Ok(Some(response.bytes()?))
    }

    fn read_file(&self, package: &PackageName, file_path: &str) -> Result<Option<String>> {
        let Some(response) = self.request_file(package, file_path)? else {
            return Ok(None);
        };

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

        let url = match Url::parse(&repository.url) {
            Ok(url) => url.to_string(),
            Err(e) => {
                debug!("Invalid repository url\n{}", e.to_string());
                return None;
            },
        };

        Some(Self { url })
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

    /// Requests a file from the specified package. Returns a `Response` if the request was successful.
    /// `RepositoryError::UnsuccessfulRequest` is returned in case of failure. If the given `file_path`
    /// escapes the parent directory `RepositoryError::EscapeDirectoryError` is returned.
    /// If the response status is 404 `None` is returned.
    fn request_file(&self, package: &PackageName, file_path: &str) -> Result<Option<Response>> {
        let parent = format!("{}/packages/{package}", self.url);
        let path = Url::parse(&format!("{parent}/{file_path}"))?.to_string();
        if !path.starts_with(&parent) {
            return Err(RepositoryError::EscapeDirectoryError(file_path.to_string()));
        }

        let response = requests::get(path)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Return an error if something went wrong with the request (apart from not found error)
        if !response.status().is_success() {
            return Err(RepositoryError::UnsuccessfulRequest(response.status()));
        }

        Ok(Some(response))
    }
}
