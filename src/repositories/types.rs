use std::collections::HashMap;

use serde::Deserialize;

/// Represents the repository metadata, containing repository information
#[derive(Deserialize, Debug)]
pub struct RepositoryMetadata {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,
}

/// Wrapper struct to deserialize package.toml files correctly
#[derive(Deserialize, Debug)]
pub(super) struct PackageMetadata {
    pub package: Package
}

/// Represents the package metadata, containing package information
#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub latest_version: String,
}

/// Represents the package version metadata, containing dependencies and targets
#[derive(Deserialize, Debug)]
pub struct PackageVersion {
    pub version: String,
    pub dependencies: Vec<String>,
    pub targets: HashMap<String, PackageTarget>,
}

/// Represents the package target data, containing the download url and installer type
#[derive(Deserialize, Debug)]
pub struct PackageTarget {
    pub url: String,
    pub installer_type: String, //TODO: change this to use installer type enum

    #[serde(default)]
    pub dependencies: Vec<String>,
}