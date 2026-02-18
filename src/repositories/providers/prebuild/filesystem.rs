use std::path::PathBuf;

use crate::{
    installer::types::Version,
    repositories::{provider::PrebuildProvider, types::Checksum},
};

pub const FILESYSTEM_PREBUILD_PROVIDER_ID: &str = "fs";

/// The filesystem prebuild provider, reading prebuild packages from a local repository on the filesystem.
pub struct FileSystemPrebuildProvider {
    path: PathBuf,
}

impl PrebuildProvider for FileSystemPrebuildProvider {
    fn get_prebuild_url(&self, package: &str, version: &Version, revision: usize, target: &str) -> Option<String> {
        todo!()
    }

    fn get_prebuild_checksum(&self, package: &str, version: &Version, revision: usize, target: &str) -> Option<Checksum> {
        todo!()
    }
}

impl FileSystemPrebuildProvider {
    /// Creates a new filesystem prebuild provider for the given url.
    /// Returns None if the url is invalid
    pub fn from_url(url: &str) -> Option<Self> {
        Some(Self { path: PathBuf::from(url) })
    }
}
