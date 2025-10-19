use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RepositoryMetadata {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct PackageMetadata {
    pub package: Package
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub latest_version: String,
}

#[derive(Deserialize, Debug)]
pub struct PackageVersion {
    pub version: String,
    pub dependencies: Vec<String>,
    pub targets: HashMap<String, PackageTarget>,
}

#[derive(Deserialize, Debug)]
pub struct PackageTarget {
    pub url: String,
    pub installer_type: String, //TODO: change this to use installer type enum

    #[serde(default)]
    pub dependencies: Vec<String>,
}