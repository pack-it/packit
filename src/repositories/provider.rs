use crate::{config::Repository, repositories::{default::{DefaultProvider, DEFAULT_PROVIDER_ID}, error::Result, filesystem::{FileSystemProvider, FILESYSTEM_PROVIDER_ID}, types::{Package, PackageVersion, RepositoryMetadata}}};

pub trait RepositoryProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata>;
    fn read_package(&self, package: String) -> Result<Package>;
    fn read_package_version(&self, package: String, version: String) -> Result<PackageVersion>;
}

pub fn create_repository_provider(repository: &Repository) -> Option<Box<dyn RepositoryProvider>> {
    match repository.provider.as_str() {
        FILESYSTEM_PROVIDER_ID => boxed(FileSystemProvider::from_repository(repository)),
        DEFAULT_PROVIDER_ID => boxed(DefaultProvider::from_repository(repository)),
        _ => None
    }
}

fn boxed(provider: Option<impl RepositoryProvider + 'static>) -> Option<Box<dyn RepositoryProvider>> {
    provider.map(|provider| Box::new(provider) as Box<dyn RepositoryProvider>)
}
