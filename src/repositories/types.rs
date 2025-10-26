use std::collections::HashMap;

use serde::Deserialize;

/// Represents the repository metadata, containing repository information.
#[derive(Deserialize, Debug)]
pub struct RepositoryMetadata {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,
}

/// Represents the package metadata, containing package information.
#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub versions: Vec<String>,
    pub latest_versions: HashMap<String, String>,
}

/// Represents the package version metadata, containing dependencies and targets.
#[derive(Deserialize, Debug)]
pub struct PackageVersion {
    pub version: String,
    pub dependencies: Vec<String>,
    pub build_dependencies: Vec<String>,
    pub targets: HashMap<String, PackageTarget>,
}

/// Represents the package target data, containing the download url and installer type.
#[derive(Deserialize, Debug)]
pub struct PackageTarget {
    pub url: String,

    #[serde(default)]
    pub dependencies: Vec<String>,

    #[serde(default)]
    pub build_dependencies: Vec<String>,
    pub build_script: Option<String>,
}
