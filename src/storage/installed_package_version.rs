use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{config, installer::types::PackageId, storage::error::InstalledPackagesError};

/// Represents a package which is installed on the system.
#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledPackageVersion {
    pub package_id: PackageId,
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub source_repository_url: String,

    #[serde(default = "config::default_repository_provider")]
    #[serde(skip_serializing_if = "is_repository_provider_default")]
    pub source_repository_provider: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub dependencies: HashSet<PackageId>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub dependents: HashSet<PackageId>,

    pub install_path: PathBuf,
    pub symlinked: bool,
    pub active: bool,
}

fn is_repository_provider_default(value: &String) -> bool {
    *value == config::default_repository_provider()
}
