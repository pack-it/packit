// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf};

use bytes::Bytes;

use crate::{
    installer::{types::PackageId, unpack::ArchiveExtension},
    repositories::{
        error::{RepositoryError, Result},
        provider::PrebuildProvider,
        types::PrebuildFileMeta,
    },
    utils::ioerror::IOResultExt,
};

pub const FILESYSTEM_PREBUILD_PROVIDER_ID: &str = "fs";

/// The filesystem prebuild provider, reading prebuild packages from a local repository on the filesystem.
pub struct FileSystemPrebuildProvider {
    path: PathBuf,
}

impl PrebuildProvider for FileSystemPrebuildProvider {
    fn get_prebuild_meta(&self, package_id: &PackageId, revision: u64, prebuild_id: &str) -> Result<PrebuildFileMeta> {
        let path = self.get_file_path(package_id, revision, prebuild_id, "toml")?;

        let metadata_string = fs::read_to_string(&path).err_with_path("read", path)?;

        Ok(toml::de::from_str(&metadata_string)?)
    }

    fn read_prebuild(&self, package_id: &PackageId, revision: u64, prebuild_id: &str) -> Result<(ArchiveExtension, Bytes)> {
        let url = self.get_file_path(package_id, revision, prebuild_id, "tar.gz")?;

        // TODO: improve efficiency of file reading
        let bytes = fs::read(&url).err_with_path("read", &url)?;

        Ok((ArchiveExtension::from_path(&url.display().to_string()), Bytes::from(bytes)))
    }
}

impl FileSystemPrebuildProvider {
    /// Creates a new filesystem prebuild provider for the given url.
    /// Returns `None` if the url is invalid
    pub fn from_url(url: &str) -> Option<Self> {
        Some(Self { path: PathBuf::from(url) })
    }

    fn get_file_path(&self, package_id: &PackageId, revision: u64, prebuild_id: &str, extension: &str) -> Result<PathBuf> {
        let prefix = package_id.name.get_prefix().to_string();
        let prebuild_name = format!("{package_id}-{revision}-{prebuild_id}.{extension}");
        let path = self.path.join("packages").join(prefix).join(&package_id.name).join(package_id.version.to_string()).join(prebuild_name);

        // Check if prebuild path exists
        if !fs::exists(&path).err_with_path("check existance of", &path)? {
            return Err(RepositoryError::PrebuildNotFound {
                prebuild_id: prebuild_id.into(),
                package_id: package_id.clone(),
                revision,
            });
        }

        Ok(path)
    }
}
