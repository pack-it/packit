use std::{fs, path::PathBuf, str::FromStr};

use crate::{
    installer::types::PackageId,
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
    fn get_prebuild_url(&self, package_id: &PackageId, revision: u64, target: &str) -> Option<String> {
        let prefix = package_id.name.chars().next()?.to_string();
        let prebuild_name = format!("{package_id}-{revision}-{target}.tar.gz");

        let path = self.path.join("packages").join(prefix).join(&package_id.name).join(package_id.version.to_string()).join(prebuild_name);

        path.as_os_str().to_str().map(|x| x.into())
    }

    fn get_prebuild_checksum(&self, package_id: &PackageId, revision: u64, target: &str) -> Result<Option<Checksum>> {
        let prefix = package_id.name.chars().next().ok_or(RepositoryError::EmptyPackageName)?.to_string();
        let prebuild_name = format!("{package_id}-{revision}-{target}.sha256");

        let path = self.path.join("packages").join(prefix).join(&package_id.name).join(package_id.version.to_string()).join(prebuild_name);

        if !fs::exists(&path)? {
            return Ok(None);
        }

        let checksum_string = fs::read_to_string(path)?;

        Ok(Some(Checksum::from_str(&checksum_string)?))
    }
}

impl FileSystemPrebuildProvider {
    /// Creates a new filesystem prebuild provider for the given url.
    /// Returns None if the url is invalid
    pub fn from_url(url: &str) -> Option<Self> {
        Some(Self { path: PathBuf::from(url) })
    }
}
