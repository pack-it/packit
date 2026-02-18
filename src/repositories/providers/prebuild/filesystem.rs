use std::{fs, path::PathBuf};

use crate::{
    installer::types::Version,
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
    fn get_prebuild_url(&self, package: &str, version: &Version, revision: usize, target: &str) -> Option<String> {
        let prefix = package.chars().next()?.to_string();
        let prebuild_name = format!("{package}@{version}-{revision}-{target}.tar.gz");

        let path = self.path.join("packages").join(prefix).join(package).join(version.to_string()).join(prebuild_name);

        path.as_os_str().to_str().map(|x| x.into())
    }

    fn get_prebuild_checksum(&self, package: &str, version: &Version, revision: usize, target: &str) -> Result<Option<Checksum>> {
        let prefix = package.chars().next().ok_or(RepositoryError::EmptyPackageName)?.to_string();
        let prebuild_name = format!("{package}@{version}-{revision}-{target}.sha256");

        let path = self.path.join("packages").join(prefix).join(package).join(version.to_string()).join(prebuild_name);

        if !fs::exists(&path)? {
            return Ok(None);
        }

        let file_bytes = fs::read(path)?;
        let sha256 = file_bytes.try_into().map_err(|_| RepositoryError::ChecksumParseError)?;

        Ok(Some(Checksum { sha256 }))
    }
}

impl FileSystemPrebuildProvider {
    /// Creates a new filesystem prebuild provider for the given url.
    /// Returns None if the url is invalid
    pub fn from_url(url: &str) -> Option<Self> {
        Some(Self { path: PathBuf::from(url) })
    }
}
