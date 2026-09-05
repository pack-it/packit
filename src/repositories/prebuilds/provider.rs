// SPDX-License-Identifier: GPL-3.0-only
use std::io::Read;

use crate::{
    config::Repository,
    installer::{types::PackageId, unpack::ArchiveExtension},
    repositories::{
        error::Result,
        prebuilds::{
            DEFAULT_PREBUILD_PROVIDER_ID, FILESYSTEM_PREBUILD_PROVIDER_ID, WEB_PREBUILD_PROVIDER_ID,
            filesystem::FileSystemPrebuildProvider, web::WebPrebuildProvider,
        },
        types::PrebuildFileMeta,
    },
};

/// Generic prebuild repository provider trait, reading prebuild packages from a repository.
pub trait PrebuildProviderImpl {
    fn get_prebuild_meta(&self, package_id: &PackageId, revision: u64, prebuild_id: &str) -> Result<PrebuildFileMeta>;

    fn read_prebuild(&self, package_id: &PackageId, revision: u64, prebuild_id: &str) -> Result<(ArchiveExtension, Box<dyn Read>)>;
}

/// Generic prebuild repository provider, managing reading prebuild packages from a repository.
pub struct PrebuildProvider {
    inner: Box<dyn PrebuildProviderImpl>,
}

impl PrebuildProvider {
    /// Creates a new `PrebuildProvider` from the given `Repository` configuration.
    /// Returns `None` if no provider can be created.
    pub fn create_from_repository(repository: &Repository) -> Option<Self> {
        let Some(url) = &repository.prebuilds_url else {
            return None;
        };

        let provider = repository.prebuilds_provider.as_deref().unwrap_or(DEFAULT_PREBUILD_PROVIDER_ID);

        let inner = match provider {
            FILESYSTEM_PREBUILD_PROVIDER_ID => boxed(FileSystemPrebuildProvider::from_url(url)),
            WEB_PREBUILD_PROVIDER_ID => boxed(WebPrebuildProvider::from_url(url)),
            _ => None,
        };

        inner.map(|inner| Self { inner })
    }

    /// Gets the metadata of a prebuild package, returns `None` if the prebuild package does not exist.
    pub fn get_prebuild_meta(&self, package_id: &PackageId, revision: u64, prebuild_id: &str) -> Result<PrebuildFileMeta> {
        self.inner.get_prebuild_meta(package_id, revision, prebuild_id)
    }

    /// Reads the prebuild package as bytes, returns a tuple containing the origin url and the bytes.
    pub fn read_prebuild(&self, package_id: &PackageId, revision: u64, prebuild_id: &str) -> Result<(ArchiveExtension, Box<dyn Read>)> {
        self.inner.read_prebuild(package_id, revision, prebuild_id)
    }
}

/// Maps an `Option<PrebuildProviderImpl>` to `Option<Box<PrebuildProviderImpl>>`.
fn boxed(provider: Option<impl PrebuildProviderImpl + 'static>) -> Option<Box<dyn PrebuildProviderImpl>> {
    provider.map(|provider| Box::new(provider) as Box<dyn PrebuildProviderImpl>)
}
