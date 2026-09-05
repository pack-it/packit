// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;

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

        inner.map(|inner| Self { inner })
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
        self.inner.read_package(package)
    }

    /// Reads the metadata of a certain version of a package, containing dependencies and targets.
    pub fn read_package_version(&self, package: &PackageName, version: &Version) -> Result<PackageVersionMeta> {
        self.inner.read_package_version(package, version)
    }

    /// Reads the list of prebuilds that can be generated for the given version of the package.
    /// Returns `None` if the prebuilds list does not exist.
    pub fn read_prebuilds_list(&self, package: &PackageName, version: &Version) -> Result<Option<PrebuildsList>> {
        self.inner.read_prebuilds_list(package, version)
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
