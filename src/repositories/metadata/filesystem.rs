// SPDX-License-Identifier: GPL-3.0-only
use std::{
    fs,
    path::{Path, PathBuf},
};

use bytes::Bytes;

use crate::{
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::{RepositoryError, Result},
        metadata::provider::MetadataProviderImpl,
        types::{IndexMeta, PackageMeta, PackageVersionMeta, PrebuildsList, RepositoryMeta},
    },
    utils::{io, ioerror::IOResultExt},
};

pub const FILESYSTEM_METADATA_PROVIDER_ID: &str = "fs";

/// The filesystem repository provider, reading package metadata from a local repository on the filesystem.
pub struct FileSystemMetadataProvider {
    path: PathBuf,
}

impl MetadataProviderImpl for FileSystemMetadataProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMeta> {
        let data = Self::read_file_string(&self.path.join("repository.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_index_metadata(&self) -> Result<IndexMeta> {
        let data = Self::read_file_string(&self.path.join("index.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        let data = Self::read_file_string(&self.path.join("packages").join(package.to_string()).join("package.toml"))?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        let path = self.path.join("packages").join(package.to_string()).join(version.to_string()).join("targets.toml");
        let data = Self::read_file_string(&path)?;

        Ok(toml::de::from_str(&data)?)
    }

    fn read_prebuilds_list(&self, package: &PackageName, version: &Version) -> Result<Option<PrebuildsList>> {
        let path = self.path.join("packages").join(package.to_string()).join(version.to_string()).join("prebuilds.toml");
        if !path.exists() {
            return Ok(None);
        }

        let data = Self::read_file_string(&path)?;
        Ok(Some(toml::de::from_str(&data)?))
    }

    fn read_file_bytes(&self, package: &PackageName, file_path: &str) -> Result<Option<Bytes>> {
        let parent = self.path.join("packages").join(package.to_string());
        let complete_path = io::normalize_path(&parent.join(file_path));
        if !complete_path.starts_with(parent) {
            return Err(RepositoryError::EscapeDirectoryError(file_path.to_string()));
        }

        if !fs::exists(&complete_path).err_with_path("check existence of", &complete_path)? {
            return Ok(None);
        }

        Ok(Some(fs::read(&complete_path).err_with_path("read", &complete_path)?.into()))
    }

    fn read_file(&self, package: &PackageName, file_path: &str) -> Result<Option<String>> {
        let parent = self.path.join("packages").join(package.to_string());
        let complete_path = io::normalize_path(&parent.join(file_path));
        if !complete_path.starts_with(parent) {
            return Err(RepositoryError::EscapeDirectoryError(file_path.to_string()));
        }

        if !fs::exists(&complete_path).err_with_path("check existence of", &complete_path)? {
            return Ok(None);
        }

        Ok(Some(Self::read_file_string(&complete_path)?))
    }
}

impl FileSystemMetadataProvider {
    /// Creates a new filesystem repository provider for the given repository.
    /// Returns `None` if the repository is not of the correct type.
    pub fn from_repository(repository: &Repository) -> Option<Self> {
        if repository.provider != FILESYSTEM_METADATA_PROVIDER_ID {
            return None;
        }

        Some(Self {
            path: io::normalize_path(&PathBuf::from(&repository.url)),
        })
    }

    /// Reads the file at the given path into a string.
    fn read_file_string(path: &Path) -> Result<String> {
        Ok(fs::read_to_string(path).err_with_path("read", path)?)
    }
}
