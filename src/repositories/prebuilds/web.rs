// SPDX-License-Identifier: GPL-3.0-only
use std::str::FromStr;

use bytes::Bytes;
use reqwest::{StatusCode, blocking::Response};
use url::Url;

use crate::{
    installer::{types::PackageId, unpack::ArchiveExtension},
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        provider::PrebuildProvider,
        types::Checksum,
    },
    utils::requests,
};

pub const WEB_PREBUILD_PROVIDER_ID: &str = "web";

/// The web prebuild provider, reading prebuild packages from a web repository.
pub struct WebPrebuildProvider {
    url: Url,
}

impl PrebuildProvider for WebPrebuildProvider {
    fn get_prebuild_url(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<Option<String>> {
        match self.read_file_path(package_id, revision, target, "tar.gz")? {
            Some((url, _)) => Ok(Some(url.to_string())),
            None => Ok(None),
        }
    }

    fn get_prebuild_checksum(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<Option<Checksum>> {
        let response = match self.read_file_path(package_id, revision, target, "sha256")? {
            Some((_, response)) => response,
            None => return Ok(None),
        };

        let checksum_string = response.text()?;

        Ok(Some(Checksum::from_str(&checksum_string)?))
    }

    fn read_prebuild(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<(ArchiveExtension, Bytes)> {
        let (url, bytes) = match self.read_file_path(package_id, revision, target, "tar.gz")? {
            Some((url, response)) => (url, response.bytes()?),
            None => {
                return Err(RepositoryError::PrebuildNotFound {
                    package_id: package_id.clone(),
                    revision,
                });
            },
        };

        Ok((ArchiveExtension::from_path(&url.to_string()), bytes))
    }
}

impl WebPrebuildProvider {
    /// Creates a new web prebuild provider for the given url.
    /// Returns None if the url is invalid.
    pub fn from_url(url: &str) -> Option<Self> {
        // Ensure the url ends in a '/', so the join function adds the path on top of the existing path
        let url = Url::from_str(&format!("{url}/")).ok()?;

        Some(Self { url })
    }

    /// Reads a file from the repository and return the url and response.
    /// Returns None if the file cannot be found in the repository.
    /// Returns a `UrlParseError` if the created url cannot be parsed.
    fn read_file_path(&self, package_id: &PackageId, revision: u64, target: &Target, extension: &str) -> Result<Option<(Url, Response)>> {
        let prefix = package_id.name.chars().next().ok_or(RepositoryError::EmptyPackageName)?.to_string();
        let target = target.architecture.to_string();
        let package_name = &package_id.name;
        let file_name = format!("{package_id}-{revision}-{target}.{extension}");

        let path = format!("packages/{target}/{prefix}/{package_name}/{file_name}");
        let url = self.url.join(&path)?;

        let response = requests::get(url.clone())?;

        // Check if the url exists
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Return an error if something went wrong with the request (apart from not found error)
        if !response.status().is_success() {
            return Err(RepositoryError::UnsuccessfulRequest(response.status()));
        }

        Ok(Some((url, response)))
    }
}
