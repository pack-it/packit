use crate::{
    config::Repository,
    installer::types::Version,
    repositories::{
        default::{DefaultProvider, DEFAULT_PROVIDER_ID},
        error::Result,
        filesystem::{FileSystemProvider, FILESYSTEM_PROVIDER_ID},
        types::{Package, PackageVersion, RepositoryMetadata},
    },
};

/// Generic repository provider trait, reading package metadata from a repository.
pub trait RepositoryProvider {
    /// Reads repository metadata from the repository, containing information about the repository.
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata>;

    /// Reads package metadata from the repository, containing information about the package.
    fn read_package(&self, package: &str) -> Result<Package>;

    /// Reads the metadata of a certain version of a package, containing dependencies and targets.
    fn read_package_version(&self, package: &str, version: &Version) -> Result<PackageVersion>;

    /// Reads the requested script from the repository.
    fn read_script(&self, package: &str, script_path: &str) -> Result<Option<String>>;
}

/// Creates a repository provider for the given repository.
pub fn create_repository_provider(repository: &Repository) -> Option<Box<dyn RepositoryProvider>> {
    match repository.provider.as_str() {
        FILESYSTEM_PROVIDER_ID => boxed(FileSystemProvider::from_repository(repository)),
        DEFAULT_PROVIDER_ID => boxed(DefaultProvider::from_repository(repository)),
        _ => None,
    }
}

/// Maps an `Option<RepositoryProvider>` to `Option<Box<RepositoryProvider>>`.
fn boxed(provider: Option<impl RepositoryProvider + 'static>) -> Option<Box<dyn RepositoryProvider>> {
    provider.map(|provider| Box::new(provider) as Box<dyn RepositoryProvider>)
}
