use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::{
        scripts::SCRIPT_EXTENSION,
        types::{Dependency, Version},
    },
    platforms,
    repositories::{
        error::{RepositoryError, Result},
        types::{
            common::{Source, Sources},
            PackageTarget, Script,
        },
    },
};

/// Represents the package version metadata, containing dependencies and targets.
#[derive(Deserialize, Debug)]
pub struct PackageVersionMeta {
    pub version: Version,

    pub dependencies: Vec<Dependency>,
    pub build_dependencies: Vec<Dependency>,
    targets: HashMap<String, PackageTarget>,

    #[serde(rename = "source")]
    pub sources: Sources,

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
    pub fn get_source(&self, target_name: &str) -> Result<&Source> {
        match &self.sources {
            Sources::Single(source) => Ok(source),
            Sources::Named(sources) => {
                let target = self.targets.get(target_name).ok_or(RepositoryError::TargetError)?;
                let source =
                    target.source.as_ref().ok_or(RepositoryError::ValidationError("Package target does not specify source".into()))?;

                Ok(sources.get(source).ok_or(RepositoryError::ValidationError("Package references an unknown source".into()))?)
            },
        }
    }

    fn get_script_path(&self, use_version_specific: bool, script: &Option<Script>, default_script_name: &str) -> String {
        match script {
            Some(Script::NameOnly(name)) => format!("{name}.{SCRIPT_EXTENSION}"),
            Some(Script::Expanded { name, version_specific }) if *version_specific => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            Some(Script::Expanded { name, .. }) => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            None if use_version_specific => format!("{}/{default_script_name}.{SCRIPT_EXTENSION}", self.version),
            None => format!("{default_script_name}.{SCRIPT_EXTENSION}"),
        }
    }

    pub fn has_target(&self, target_name: &str) -> Result<bool> {
        match self.get_target(target_name) {
            Ok(_) => Ok(true),
            Err(RepositoryError::TargetError) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn get_target(&self, target_name: &str) -> Result<&PackageTarget> {
        // Read target specific target
        if let Some(target) = self.targets.get(target_name) {
            return Ok(target);
        }

        // Read OS group target
        if let Some(target) = self.targets.get(platforms::get_os_name(target_name)) {
            return Ok(target);
        }

        // If the platform is unix, reade the unix target
        if platforms::is_unix(target_name) {
            if let Some(target) = self.targets.get("unix") {
                return Ok(target);
            }
        }

        Err(RepositoryError::TargetError)
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

    /// Checks if there are conflicts in the package version metadata
    pub fn has_conflicts(&self) -> bool {
        // Check if a global dependency is also specified as target specific dependency
        for dependency in &self.dependencies {
            for (_, target) in &self.targets {
                for target_dependency in &target.dependencies {
                    if dependency.get_name() == target_dependency.get_name() {
                        return true;
                    }
                }
            }
        }

        // Check if a global build dependency is also specified as target specific build dependency
        for dependency in &self.build_dependencies {
            for (_, target) in &self.targets {
                for target_dependency in &target.build_dependencies {
                    if dependency.get_name() == target_dependency.get_name() {
                        return true;
                    }
                }
            }
        }

        // If we have a single source, we don't allow referencing sources in the targets
        if let Sources::Single(_) = self.sources {
            if self.targets.iter().any(|(_, target)| target.source.is_some()) {
                return true;
            }
        }

        // If we have named sources, we check for valid referencing in the targets
        if let Sources::Named(sources) = &self.sources {
            for (_, target) in &self.targets {
                match &target.source {
                    Some(source) if !sources.contains_key(source) => return true,
                    None => return true,
                    _ => continue,
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
