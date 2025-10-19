use crate::repositories::{error::Result, provider::RepositoryProvider, types::{Package, PackageMetadata, PackageVersion, RepositoryMetadata}};

pub fn read_repository_metadata(provider: &impl RepositoryProvider) -> Result<RepositoryMetadata> {
    let data = provider.read_file("/repository.toml".into())?;

    Ok(toml::de::from_str(&data)?)
}

pub fn read_package(provider: &impl RepositoryProvider, package: String) -> Result<Package> {
    let data = provider.read_file(format!("/packages/{package}/package.toml"))?;

    let content: PackageMetadata = toml::de::from_str(&data)?;

    Ok(content.package)
}

pub fn read_package_version(provider: &impl RepositoryProvider, package: String, version: String) -> Result<PackageVersion> {
    let data = provider.read_file(format!("/packages/{package}/{version}/targets.toml"))?;

    Ok(toml::de::from_str(&data)?)
}
