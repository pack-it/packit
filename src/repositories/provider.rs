use bytes::Bytes;

use crate::{
    config::Repository,
    installer::types::{PackageId, Version},
    repositories::{
        error::Result,
        metadata::{FileSystemMetadataProvider, WebMetadataProvider, FILESYSTEM_METADATA_PROVIDER_ID, WEB_METADATA_PROVIDER_ID},
        prebuilds::{FileSystemPrebuildProvider, DEFAULT_PREBUILD_PROVIDER_ID, FILESYSTEM_PREBUILD_PROVIDER_ID},
        types::{Checksum, PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
};

/// Generic metadata repository provider trait, reading package metadata from a repository.
pub trait MetadataProvider {
    /// Reads repository metadata from the repository, containing information about the repository.
    fn read_repository_metadata(&self) -> Result<RepositoryMeta>;

    /// Reads package metadata from the repository, containing information about the package.
    fn read_package(&self, package: &str) -> Result<PackageMeta>;

    /// Reads the metadata of a certain version of a package, containing dependencies and targets.
    fn read_package_version(&self, package: &str, version: &Version) -> Result<PackageVersionMeta>;

    /// Reads the requested script from the repository.
    fn read_script(&self, package: &str, script_path: &str) -> Result<Option<String>>;
}

/// Generic prebuild repository provider trait, reading prebuild packages from a repository.
pub trait PrebuildProvider {
    /// Gets the url of a prebuild package, returns None if the prebuild package does not exist.
    fn get_prebuild_url(&self, package_id: &PackageId, revision: u64, target: &str) -> Result<Option<String>>;

    /// Gets the checksum of a prebuild package, returns None if the prebuild package does not exist.
    fn get_prebuild_checksum(&self, package_id: &PackageId, revision: u64, target: &str) -> Result<Option<Checksum>>;

    /// Reads the prebuild package as bytes, returns a tuple containing the origin url and the bytes.
    fn read_prebuild(&self, package_id: &PackageId, revision: u64, target: &str) -> Result<(String, Bytes)>;
}

/// Creates a metadata repository provider for the given repository.
pub fn create_metadata_provider(repository: &Repository) -> Option<Box<dyn MetadataProvider>> {
    match repository.provider.as_str() {
        FILESYSTEM_METADATA_PROVIDER_ID => boxed(FileSystemMetadataProvider::from_repository(repository)),
        WEB_METADATA_PROVIDER_ID => boxed(WebMetadataProvider::from_repository(repository)),
        _ => None,
    }
}

/// Creates a prebuild repository provider for the given repository.
pub fn create_prebuild_provider(repository: &Repository, repo_metadata: RepositoryMeta) -> Option<Box<dyn PrebuildProvider>> {
    let (url, provider) = get_prebuild_repository_info(repository, repo_metadata)?;

    match provider.as_str() {
        FILESYSTEM_PREBUILD_PROVIDER_ID => boxed_prebuild(FileSystemPrebuildProvider::from_url(&url)),
        _ => None,
    }
}

fn get_prebuild_repository_info(repository: &Repository, repo_metadata: RepositoryMeta) -> Option<(String, String)> {
    if let Some(url) = &repository.prebuilds_url {
        let provider = repository.prebuilds_provider.clone().unwrap_or(DEFAULT_PREBUILD_PROVIDER_ID.into());

        return Some((url.clone(), provider));
    }

    if let Some(url) = &repo_metadata.prebuilds_url {
        let provider = repo_metadata.prebuilds_provider.clone().unwrap_or(DEFAULT_PREBUILD_PROVIDER_ID.into());

        return Some((url.clone(), provider));
    }

    None
}

/// Maps an `Option<MetadataProvider>` to `Option<Box<MetadataProvider>>`.
fn boxed(provider: Option<impl MetadataProvider + 'static>) -> Option<Box<dyn MetadataProvider>> {
    provider.map(|provider| Box::new(provider) as Box<dyn MetadataProvider>)
}

/// Maps an `Option<PrebuildProvider>` to `Option<Box<PrebuildProvider>>`.
fn boxed_prebuild(provider: Option<impl PrebuildProvider + 'static>) -> Option<Box<dyn PrebuildProvider>> {
    provider.map(|provider| Box::new(provider) as Box<dyn PrebuildProvider>)
}
