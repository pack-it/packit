// SPDX-License-Identifier: GPL-3.0-only
use std::str::FromStr;

use bytes::Bytes;
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
        match self.get_file_path(package_id, revision, target, "tar.gz")? {
            Some(url) => Ok(Some(url.to_string())),
            None => Ok(None),
        }
    }

    fn get_prebuild_checksum(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<Option<Checksum>> {
        let url = match self.get_file_path(package_id, revision, target, "sha256")? {
            Some(path) => path,
            None => return Ok(None),
        };

        let checksum_string = requests::get(url)?.text()?;

        Ok(Some(Checksum::from_str(&checksum_string)?))
    }

    fn read_prebuild(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<(ArchiveExtension, Bytes)> {
        let url = self.get_prebuild_url(package_id, revision, target)?.ok_or(RepositoryError::PrebuildNotFound {
            package_id: package_id.clone(),
            revision,
        })?;

        let bytes = requests::get(&url)?.bytes()?;

        Ok((ArchiveExtension::from_path(&url), bytes))
    }
}

impl WebPrebuildProvider {
    /// Creates a new filesystem prebuild provider for the given url.
    /// Returns None if the url is invalid.
    pub fn from_url(url: &str) -> Option<Self> {
        let url = Url::from_str(url).ok()?;
        Some(Self { url })
    }

    /// Gets the url for a file. Note that it doesn't check if the url exists.
    /// Returns a `UrlParseError` if the created url cannot be parsed.
    fn get_file_path(&self, package_id: &PackageId, revision: u64, target: &Target, extension: &str) -> Result<Option<Url>> {
        let prefix = package_id.name.chars().next().ok_or(RepositoryError::EmptyPackageName)?.to_string();
        let target = target.architecture.to_string();
        let prebuild_name = format!("{package_id}-{revision}-{target}.{extension}");
        let url = self
            .url
            .join(&target)?
            .join("packages")?
            .join(&prefix)?
            .join(&package_id.name)?
            .join(&package_id.version.to_string())?
            .join(&prebuild_name)?;

        Ok(Some(url))
    }
}
