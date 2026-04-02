// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf, str::FromStr};

use bytes::Bytes;

use crate::{
    installer::{types::PackageId, unpack::ArchiveExtension},
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        provider::PrebuildProvider,
        types::Checksum,
    },
};

pub const FILESYSTEM_PREBUILD_PROVIDER_ID: &str = "fs";

/// The filesystem prebuild provider, reading prebuild packages from a local repository on the filesystem.
pub struct FileSystemPrebuildProvider {
    path: PathBuf,
}

impl PrebuildProvider for FileSystemPrebuildProvider {
    fn get_prebuild_url(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<Option<String>> {
        match self.get_file_path(package_id, revision, target, "tar.gz")? {
            Some(path) => Ok(path.as_os_str().to_str().map(|x| x.into())),
            None => Ok(None),
        }
    }

    fn get_prebuild_checksum(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<Option<Checksum>> {
        let path = match self.get_file_path(package_id, revision, target, "sha256")? {
            Some(path) => path,
            None => return Ok(None),
        };

        let checksum_string = fs::read_to_string(path)?;

        Ok(Some(Checksum::from_str(&checksum_string)?))
    }

    fn read_prebuild(&self, package_id: &PackageId, revision: u64, target: &Target) -> Result<(ArchiveExtension, Bytes)> {
        let url = self.get_prebuild_url(package_id, revision, target)?.ok_or(RepositoryError::PrebuildNotFound {
            package_id: package_id.clone(),
            revision,
        })?;

        // TODO: improve efficiency of file reading
        let bytes = fs::read(&url)?;

        Ok((ArchiveExtension::from_path(&url), Bytes::from(bytes)))
    }
}

impl FileSystemPrebuildProvider {
    /// Creates a new filesystem prebuild provider for the given url.
    /// Returns None if the url is invalid
    pub fn from_url(url: &str) -> Option<Self> {
        Some(Self { path: PathBuf::from(url) })
    }

    fn get_file_path(&self, package_id: &PackageId, revision: u64, target: &Target, extension: &str) -> Result<Option<PathBuf>> {
        let prefix = package_id.name.chars().next().ok_or(RepositoryError::EmptyPackageName)?.to_string();
        let target = target.architecture.to_string();
        let prebuild_name = format!("{package_id}-{revision}-{target}.{extension}");
        let path = self.path.join("packages").join(prefix).join(&package_id.name).join(package_id.version.to_string()).join(prebuild_name);

        if !fs::exists(&path)? {
            return Ok(None);
        }

        Ok(Some(path))
    }
}
