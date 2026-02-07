use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::{
        scripts::SCRIPT_EXTENSION,
        types::{Dependency, Version},
    },
    repositories::error::{RepositoryError, Result},
    utils::checksum::Checksum,
};

/// Represents the repository metadata, containing repository information.
#[derive(Deserialize, Debug)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,
}

/// Represents the package metadata, containing package information.
#[derive(Deserialize, Debug)]
pub struct PackageMeta {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub versions: Vec<Version>,
    pub latest_versions: HashMap<String, Version>,
}

impl PackageMeta {
    pub fn get_latest_version(&self, target_name: &str) -> Result<&Version> {
        Ok(self.latest_versions.get(target_name).ok_or(RepositoryError::TargetError)?)
    }
}

/// Represents the package version metadata, containing dependencies and targets.
#[derive(Deserialize, Debug)]
pub struct PackageVersionMeta {
    pub version: Version,

    pub dependencies: Vec<Dependency>,
    pub build_dependencies: Vec<Dependency>,
    pub targets: HashMap<String, PackageTarget>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    #[serde(default = "PackageVersionMeta::default_skip_symlinking")]
    pub skip_symlinking: bool,

    #[serde(default)]
    pub script_args: HashMap<String, String>,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    pub use_version_specific_build: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    pub use_version_specific_preinstall: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    pub use_version_specific_postinstall: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    pub use_version_specific_test: bool,
}

impl PackageVersionMeta {
    fn get_script_path(&self, use_version_specific: bool, script: &Option<Script>, default_script_name: &str) -> String {
        match script {
            Some(Script::NameOnly(name)) => format!("{name}.{SCRIPT_EXTENSION}"),
            Some(Script::Expanded { name, version_specific }) if *version_specific => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            Some(Script::Expanded { name, .. }) => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            None if use_version_specific => format!("{}/{default_script_name}.{SCRIPT_EXTENSION}", self.version),
            None => format!("{default_script_name}.{SCRIPT_EXTENSION}"),
        }
    }

    pub fn get_target(&self, target_name: &str) -> Result<&PackageTarget> {
        Ok(self.targets.get(target_name).ok_or(RepositoryError::TargetError)?)
    }

    pub fn get_build_script_path(&self, target_name: &str) -> Result<String> {
        let target = self.get_target(target_name)?;

        Ok(self.get_script_path(self.use_version_specific_build, &target.build_script, "build"))
    }

    pub fn get_preinstall_script_path(&self, target_name: &str) -> Result<String> {
        let target = self.get_target(target_name)?;

        Ok(self.get_script_path(self.use_version_specific_preinstall, &target.preinstall_script, "preinstall"))
    }

    pub fn get_postinstall_script_path(&self, target_name: &str) -> Result<String> {
        let target = self.get_target(target_name)?;

        Ok(self.get_script_path(self.use_version_specific_postinstall, &target.postinstall_script, "postinstall"))
    }

    pub fn get_test_script_path(&self, target_name: &str) -> Result<String> {
        let target = self.get_target(target_name)?;

        Ok(self.get_script_path(self.use_version_specific_test, &target.test_script, "test"))
    }

    /// Gets the script arguments for the given target.
    /// Returns None when the target cannot be found.
    pub fn get_script_args(&self, target_name: &str) -> Result<HashMap<&str, &str>> {
        let target = self.get_target(target_name)?;

        Ok(self.script_args.iter().chain(target.script_args.iter()).map(|(key, value)| (key.as_str(), value.as_str())).collect())
    }

    /// Checks if there are conflicts between the global and target specific dependencies
    pub fn has_conflicts(&self) -> bool {
        for dependency in &self.dependencies {
            for (_, target) in &self.targets {
                for target_dependency in &target.dependencies {
                    if dependency.get_name() == target_dependency.get_name() {
                        return true;
                    }
                }
            }
        }

        for dependency in &self.build_dependencies {
            for (_, target) in &self.targets {
                for target_dependency in &target.build_dependencies {
                    if dependency.get_name() == target_dependency.get_name() {
                        return true;
                    }
                }
            }
        }

        false
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
    pub checksum: Checksum,

    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    #[serde(default)]
    pub build_dependencies: Vec<Dependency>,

    pub skip_symlinking: Option<bool>,

    #[serde(default)]
    pub script_args: HashMap<String, String>,

    pub build_script: Option<Script>,
    pub preinstall_script: Option<Script>,
    pub postinstall_script: Option<Script>,
    pub test_script: Option<Script>,
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
