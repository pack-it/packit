use crate::{config::Repository, repositories::{error::Result, filesystem::FileSystemProvider, types::{Package, PackageVersion, RepositoryMetadata}}};

pub trait RepositoryProvider {
    fn read_repository_metadata(&self) -> Result<RepositoryMetadata>;
    fn read_package(&self, package: String) -> Result<Package>;
    fn read_package_version(&self, package: String, version: String) -> Result<PackageVersion>;
}

pub fn create_repository_provider(repository: &Repository) -> Option<impl RepositoryProvider> {
    match repository.provider.as_str() {
        "fs" => FileSystemProvider::from_repository(repository),
        _ => None
    }
}
