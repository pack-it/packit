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

    #[serde(default = "default_skip_symlinking")]
    pub skip_symlinking: bool,
}

/// Represents the package target data, containing the download url and installer type.
#[derive(Deserialize, Debug)]
pub struct PackageTarget {
    pub url: String,

    #[serde(default)]
    pub dependencies: Vec<String>,

    #[serde(default)]
    pub build_dependencies: Vec<String>,
    pub skip_symlinking: Option<bool>,
    pub build_script: Option<String>,
    pub preinstall_script: Option<String>,
    pub postinstall_script: Option<String>,
}

fn default_skip_symlinking() -> bool {
    false
}
