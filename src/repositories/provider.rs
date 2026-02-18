use crate::{
    config::Repository,
    installer::types::Version,
    repositories::{
        error::Result,
        providers::{
            prebuild::{FileSystemPrebuildProvider, DEFAULT_PREBUILD_PROVIDER_ID, FILESYSTEM_PREBUILD_PROVIDER_ID},
            repository::{FileSystemProvider, WebProvider, FILESYSTEM_PROVIDER_ID, WEB_PROVIDER_ID},
        },
        types::{Checksum, PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
};

/// Generic repository provider trait, reading package metadata from a repository.
pub trait RepositoryProvider {
    /// Reads repository metadata from the repository, containing information about the repository.
    fn read_repository_metadata(&self) -> Result<RepositoryMeta>;

    /// Reads package metadata from the repository, containing information about the package.
    fn read_package(&self, package: &str) -> Result<PackageMeta>;

    /// Reads the metadata of a certain version of a package, containing dependencies and targets.
    fn read_package_version(&self, package: &str, version: &Version) -> Result<PackageVersionMeta>;

    /// Reads the requested script from the repository.
    fn read_script(&self, package: &str, script_path: &str) -> Result<Option<String>>;
}

/// Generic prebuild provider trait, reading prebuild packages from a repository.
pub trait PrebuildProvider {
    /// Gets the url of a prebuild package, returns None if the prebuild package does not exist
    fn get_prebuild_url(&self, package: &str, version: &Version, revision: usize, target: &str) -> Option<String>;

    /// Gets the checksum of a prebuild package, returns None if the prebuild package does not exist
    fn get_prebuild_checksum(&self, package: &str, version: &Version, revision: usize, target: &str) -> Option<Checksum>;
}

/// Creates a repository provider for the given repository.
pub fn create_repository_provider(repository: &Repository) -> Option<Box<dyn RepositoryProvider>> {
    match repository.provider.as_str() {
        FILESYSTEM_PROVIDER_ID => boxed(FileSystemProvider::from_repository(repository)),
        WEB_PROVIDER_ID => boxed(WebProvider::from_repository(repository)),
        _ => None,
    }
}

/// Creates a prebuild provider for the given repository.
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

/// Maps an `Option<RepositoryProvider>` to `Option<Box<RepositoryProvider>>`.
fn boxed(provider: Option<impl RepositoryProvider + 'static>) -> Option<Box<dyn RepositoryProvider>> {
    provider.map(|provider| Box::new(provider) as Box<dyn RepositoryProvider>)
}

/// Maps an `Option<PrebuildProvider>` to `Option<Box<PrebuildProvider>>`.
fn boxed_prebuild(provider: Option<impl PrebuildProvider + 'static>) -> Option<Box<dyn PrebuildProvider>> {
    provider.map(|provider| Box::new(provider) as Box<dyn PrebuildProvider>)
}
