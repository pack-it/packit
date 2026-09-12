// SPDX-License-Identifier: GPL-3.0-only
use std::{hash::Hash, sync::Arc};

use bytes::Bytes;
use moka::{Equivalent, sync::Cache};

use crate::{
    config::Repository,
    installer::types::{PackageName, Version},
    repositories::{
        error::Result,
        metadata::{
            FILESYSTEM_METADATA_PROVIDER_ID, WEB_METADATA_PROVIDER_ID, filesystem::FileSystemMetadataProvider, web::WebMetadataProvider,
        },
        types::{IndexMeta, PackageMeta, PackageVersionMeta, PrebuildsList, RepositoryMeta},
    },
};

/// Generic metadata repository provider trait, reading package metadata from a repository.
pub trait MetadataProviderImpl {
    fn read_repository_metadata(&self) -> Result<RepositoryMeta>;

    fn read_index_metadata(&self) -> Result<IndexMeta>;

    fn read_package(&self, package: &PackageName) -> Result<PackageMeta>;

    fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta>;

    fn read_prebuilds_list(&self, package: &PackageName, version: &Version) -> Result<Option<PrebuildsList>>;

    fn read_file_bytes(&self, package: &PackageName, file_path: &str) -> Result<Option<Bytes>>;

    fn read_file(&self, package: &PackageName, file_path: &str) -> Result<Option<String>>;
}

/// Generic metadata repository provider, managing the reading of metadata from a repository.
pub struct MetadataProvider {
    inner: Box<dyn MetadataProviderImpl>,
    cache: MetadataCache,
}

impl MetadataProvider {
    /// Creates a new `MetadataProvider` from the given `Repository` configuration.
    /// Returns `None` if no provider can be created.
    pub fn create_from_repository(repository: &Repository) -> Option<Self> {
        let inner = match repository.provider.as_str() {
            FILESYSTEM_METADATA_PROVIDER_ID => boxed(FileSystemMetadataProvider::from_repository(repository)),
            WEB_METADATA_PROVIDER_ID => boxed(WebMetadataProvider::from_repository(repository)),
            _ => None,
        };

        inner.map(|inner| Self {
            inner,
            cache: MetadataCache::new(),
        })
    }

    /// Reads repository metadata from the repository, containing information about the repository.
    pub fn read_repository_metadata(&self) -> Result<RepositoryMeta> {
        self.inner.read_repository_metadata()
    }

    /// Reads index metadata from the repository, containing all supported packages.
    pub fn read_index_metadata(&self) -> Result<IndexMeta> {
        self.inner.read_index_metadata()
    }

    /// Reads package metadata from the repository, containing information about the package.
    pub fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        self.cache.get_package(package, || self.inner.read_package(package))
    }

    /// Reads the metadata of a certain version of a package, containing dependencies and targets.
    pub fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        self.cache.get_package_version(package, version, || self.inner.read_package_version(package, version))
    }

    /// Reads the list of prebuilds that can be generated for the given version of the package.
    /// Returns `None` if the prebuilds list does not exist.
    pub fn read_prebuilds_list(&self, package: &PackageName, version: &Version) -> Result<Option<PrebuildsList>> {
        self.cache.get_prebuilds_list(package, version, || self.inner.read_prebuilds_list(package, version))
    }

    /// Reads the requested file from the repository as bytes.
    pub fn read_file_bytes(&self, package: &PackageName, file_path: &str) -> Result<Option<Bytes>> {
        self.inner.read_file_bytes(package, file_path)
    }

    /// Reads the requested file from the repository.
    pub fn read_file(&self, package: &PackageName, file_path: &str) -> Result<Option<String>> {
        self.inner.read_file(package, file_path)
    }
}

/// Maps an `Option<MetadataProviderImpl>` to `Option<Box<MetadataProviderImpl>>`.
fn boxed(provider: Option<impl MetadataProviderImpl + 'static>) -> Option<Box<dyn MetadataProviderImpl>> {
    provider.map(|provider| Box::new(provider) as Box<dyn MetadataProviderImpl>)
}

struct MetadataCache {
    package_cache: Cache<PackageName, PackageMeta>,
    package_version_cache: Cache<PackageVersionKey, PackageVersionMeta>,
    prebuilds_list_cache: Cache<PackageVersionKey, Option<PrebuildsList>>,
}

impl MetadataCache {
    pub fn new() -> Self {
        Self {
            package_cache: Cache::new(100),
            package_version_cache: Cache::new(100),
            prebuilds_list_cache: Cache::new(100),
        }
    }

    pub fn get_package<F>(&self, package: &PackageName, init: F) -> Result<PackageMeta>
    where
        F: FnOnce() -> Result<PackageMeta>,
    {
        self.package_cache.try_get_with_by_ref(package, init).map_err(Arc::unwrap_or_clone)
    }

    pub fn get_package_version<F>(&self, package: &PackageName, version: &Version, init: F) -> Result<PackageVersionMeta>
    where
        F: FnOnce() -> Result<PackageVersionMeta>,
    {
        // Try to get value without cloning key
        let key = PackageVersionKeyRef(package, version);
        if let Some(value) = self.package_version_cache.get(&key) {
            return Ok(value);
        }

        // Initialize or get value with cloned key, to avoid concurrent fetch calls
        let key = PackageVersionKey(package.clone(), version.clone());
        self.package_version_cache.try_get_with(key, init).map_err(Arc::unwrap_or_clone)
    }

    pub fn get_prebuilds_list<F>(&self, package: &PackageName, version: &Version, init: F) -> Result<Option<PrebuildsList>>
    where
        F: FnOnce() -> Result<Option<PrebuildsList>>,
    {
        // Try to get value without cloning key
        let key = PackageVersionKeyRef(package, version);
        if let Some(value) = self.prebuilds_list_cache.get(&key) {
            return Ok(value);
        }

        // Initialize or get value with cloned key, to avoid concurrent fetch calls
        let key = PackageVersionKey(package.clone(), version.clone());
        self.prebuilds_list_cache.try_get_with(key, init).map_err(Arc::unwrap_or_clone)
    }
}

#[derive(PartialEq, Eq)]
struct PackageVersionKey(PackageName, Version);

#[derive(Hash)]
struct PackageVersionKeyRef<'a>(&'a PackageName, &'a Version);

impl Equivalent<PackageVersionKey> for PackageVersionKeyRef<'_> {
    fn equivalent(&self, key: &PackageVersionKey) -> bool {
        self.0 == &key.0 && self.1 == &key.1
    }
}

impl Hash for PackageVersionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        PackageVersionKeyRef(&self.0, &self.1).hash(state);
    }
}
