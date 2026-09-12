// SPDX-License-Identifier: GPL-3.0-only
use std::sync::Arc;

use bytes::Bytes;
use moka::sync::Cache;
use once_cell::sync::OnceCell;

use crate::{
    config::Repository,
    installer::types::{PackageId, PackageName, Version},
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
        self.cache.get_repository_metadata(|| self.inner.read_repository_metadata())
    }

    /// Reads index metadata from the repository, containing all supported packages.
    pub fn read_index_metadata(&self) -> Result<IndexMeta> {
        self.cache.get_index_metadata(|| self.inner.read_index_metadata())
    }

    /// Reads package metadata from the repository, containing information about the package.
    pub fn read_package(&self, package: &PackageName) -> Result<PackageMeta> {
        self.cache.get_package(package, || self.inner.read_package(package))
    }

    /// Reads the metadata of a certain version of a package, containing dependencies and targets.
    pub fn read_package_version(&self, package_id: &PackageId) -> Result<PackageVersionMeta> {
        self.cache.get_package_version(package_id, || {
            self.inner.read_package_version(&package_id.name, &package_id.version)
        })
    }

    /// Reads the list of prebuilds that can be generated for the given version of the package.
    /// Returns `None` if the prebuilds list does not exist.
    pub fn read_prebuilds_list(&self, package_id: &PackageId) -> Result<Option<PrebuildsList>> {
        self.cache.get_prebuilds_list(package_id, || self.inner.read_prebuilds_list(&package_id.name, &package_id.version))
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

/// Represents the cache of the metadata for a single `MetadataProvider`.
struct MetadataCache {
    repository_metadata_cache: OnceCell<RepositoryMeta>,
    index_metadata_cache: OnceCell<IndexMeta>,
    package_cache: Cache<PackageName, PackageMeta>,
    package_version_cache: Cache<PackageId, PackageVersionMeta>,
    prebuilds_list_cache: Cache<PackageId, Option<PrebuildsList>>,
}

impl MetadataCache {
    /// Creates a new `MetadataCache`.
    pub fn new() -> Self {
        Self {
            repository_metadata_cache: OnceCell::new(),
            index_metadata_cache: OnceCell::new(),
            package_cache: Cache::new(100),
            package_version_cache: Cache::new(100),
            prebuilds_list_cache: Cache::new(100),
        }
    }

    /// Gets the `RepositoryMeta` from the cache, or initializes it using the init function.
    pub fn get_repository_metadata<F>(&self, init: F) -> Result<RepositoryMeta>
    where
        F: FnOnce() -> Result<RepositoryMeta>,
    {
        self.repository_metadata_cache.get_or_try_init(init).map(Clone::clone)
    }

    /// Gets the `IndexMeta` from the cache, or initializes it using the init function.
    pub fn get_index_metadata<F>(&self, init: F) -> Result<IndexMeta>
    where
        F: FnOnce() -> Result<IndexMeta>,
    {
        self.index_metadata_cache.get_or_try_init(init).map(Clone::clone)
    }

    /// Gets the `PackageMeta` from the cache, or initializes it using the init function.
    pub fn get_package<F>(&self, package: &PackageName, init: F) -> Result<PackageMeta>
    where
        F: FnOnce() -> Result<PackageMeta>,
    {
        self.package_cache.try_get_with_by_ref(package, init).map_err(Arc::unwrap_or_clone)
    }

    /// Gets the `PackageVersionMeta` from the cache, or initializes it using the init function.
    pub fn get_package_version<F>(&self, package_id: &PackageId, init: F) -> Result<PackageVersionMeta>
    where
        F: FnOnce() -> Result<PackageVersionMeta>,
    {
        self.package_version_cache.try_get_with_by_ref(package_id, init).map_err(Arc::unwrap_or_clone)
    }

    /// Gets the `PrebuildsList` from the cache, or initializes it using the init function.
    pub fn get_prebuilds_list<F>(&self, package_id: &PackageId, init: F) -> Result<Option<PrebuildsList>>
    where
        F: FnOnce() -> Result<Option<PrebuildsList>>,
    {
        self.prebuilds_list_cache.try_get_with_by_ref(package_id, init).map_err(Arc::unwrap_or_clone)
    }
}
