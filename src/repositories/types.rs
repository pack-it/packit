use std::collections::HashMap;

use serde::Deserialize;

use crate::installer::scripts::SCRIPT_EXTENSION;

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

    #[serde(default = "PackageVersion::default_skip_symlinking")]
    pub skip_symlinking: bool,

    #[serde(default)]
    pub script_args: HashMap<String, String>,

    #[serde(default = "PackageVersion::default_use_version_specific")]
    pub use_version_specific_preinstall: bool,

    #[serde(default = "PackageVersion::default_use_version_specific")]
    pub use_version_specific_build: bool,

    #[serde(default = "PackageVersion::default_use_version_specific")]
    pub use_version_specific_postinstall: bool,
}

impl PackageVersion {
    fn get_script_name(&self, use_version_specific: bool, script: &Option<Script>, default_script_name: &str) -> String {
        match script {
            Some(Script::NameOnly(name)) => format!("{name}.{SCRIPT_EXTENSION}"),
            Some(Script::Expanded { name, version_specific }) if *version_specific => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            Some(Script::Expanded { name, .. }) => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            None if use_version_specific => format!("{}/{default_script_name}.{SCRIPT_EXTENSION}", self.version),
            None => format!("{default_script_name}.{SCRIPT_EXTENSION}"),
        }
    }

    pub fn get_preinstall_script_name(&self, target_name: &str) -> Option<String> {
        let target = self.targets.get(target_name)?;

        Some(self.get_script_name(self.use_version_specific_preinstall, &target.preinstall_script, "preinstall"))
    }

    pub fn get_build_script_name(&self, target_name: &str) -> Option<String> {
        let target = self.targets.get(target_name)?;

        Some(self.get_script_name(self.use_version_specific_build, &target.build_script, "build"))
    }

    pub fn get_postinstall_script_name(&self, target_name: &str) -> Option<String> {
        let target = self.targets.get(target_name)?;

        Some(self.get_script_name(self.use_version_specific_postinstall, &target.postinstall_script, "postinstall"))
    }

    fn default_skip_symlinking() -> bool {
        false
    }

    fn default_use_version_specific() -> bool {
        false
    }
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

    #[serde(default)]
    pub script_args: HashMap<String, String>,

    pub build_script: Option<Script>,
    pub preinstall_script: Option<Script>,
    pub postinstall_script: Option<Script>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    NameOnly(String),
    Expanded {
        name: String,
        version_specific: bool,
    },
}
