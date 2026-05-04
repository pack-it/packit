// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    installer::{
        scripts::SCRIPT_EXTENSION,
        types::{Dependency, Version},
    },
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        types::{Licenses, PackageTarget, Script, Source, Sources, TargetBounds},
    },
};

/// Represents the package version metadata, containing dependencies and targets.
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageVersionMeta {
    pub version: Version,

    #[serde(default, skip_serializing_if = "Licenses::is_unknown")]
    pub license: Licenses,

    pub dependencies: Vec<Dependency>,
    pub build_dependencies: Vec<Dependency>,

    #[serde(default = "PackageVersionMeta::default_skip_symlinking")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_skip_symlinking")]
    pub skip_symlinking: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_use_version_specific")]
    pub use_version_specific_build: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_use_version_specific")]
    pub use_version_specific_preinstall: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_use_version_specific")]
    pub use_version_specific_postinstall: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_use_version_specific")]
    pub use_version_specific_test: bool,

    #[serde(default = "PackageVersionMeta::default_use_version_specific")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_use_version_specific")]
    pub use_version_specific_uninstall: bool,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub external_test_files: HashSet<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub script_args: HashMap<String, String>,

    #[serde(rename = "source")]
    pub sources: Sources,

    pub targets: HashMap<TargetBounds, PackageTarget>,
}

impl PackageVersionMeta {
    /// Gets the best satisfying target bound for the given target. Wraps around the `TargetBounds::get_best_target` method.
    pub fn get_best_target(&self, target: &Target) -> Result<TargetBounds> {
        match TargetBounds::get_best_target(target, self.targets.keys().collect()) {
            Some(target) => Ok(target.clone()),
            None => Err(RepositoryError::TargetError),
        }
    }

    /// Gets the `Source` for the given target bounds.
    /// Returns a `RepositoryError::ValidationError` if the target does not specify a source or if an unknown source was referenced.
    pub fn get_source(&self, target_bounds: &TargetBounds) -> Result<&Source> {
        match &self.sources {
            Sources::Single(source) => Ok(source),
            Sources::Named(sources) => {
                let target = self.get_target(target_bounds)?;
                let source =
                    target.source.as_ref().ok_or(RepositoryError::ValidationError("Package target does not specify source".into()))?;

                Ok(sources.get(source).ok_or(RepositoryError::ValidationError("Package references an unknown source".into()))?)
            },
        }
    }

    /// Gets the script path based on the `use_version_specific` parameter and given script.
    /// `use_version_specific` is only used if the script is None. The version is from `Self`.
    fn get_script_path(&self, use_version_specific: bool, script: &Option<Script>, default_script_name: &str) -> String {
        match script {
            Some(Script::NameOnly(name)) => format!("{name}.{SCRIPT_EXTENSION}"),
            Some(Script::Expanded { name, version_specific }) if *version_specific => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            Some(Script::Expanded { name, .. }) => format!("{}/{name}.{SCRIPT_EXTENSION}", self.version),
            None if use_version_specific => format!("{}/{default_script_name}.{SCRIPT_EXTENSION}", self.version),
            None => format!("{default_script_name}.{SCRIPT_EXTENSION}"),
        }
    }

    /// Gets a target with the given target bounds.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_target(&self, target_bounds: &TargetBounds) -> Result<&PackageTarget> {
        self.targets.get(target_bounds).ok_or(RepositoryError::TargetError)
    }

    /// Gets the build script path with the given target bounds.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_build_script_path(&self, target_bounds: &TargetBounds) -> Result<String> {
        let target = self.get_target(target_bounds)?;

        Ok(self.get_script_path(self.use_version_specific_build, &target.build_script, "build"))
    }

    /// Gets the pre-install script path with the given target bounds.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_preinstall_script_path(&self, target_bounds: &TargetBounds) -> Result<String> {
        let target = self.get_target(target_bounds)?;

        Ok(self.get_script_path(self.use_version_specific_preinstall, &target.preinstall_script, "preinstall"))
    }

    /// Gets the post-install script path with the given target bounds.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_postinstall_script_path(&self, target_bounds: &TargetBounds) -> Result<String> {
        let target = self.get_target(target_bounds)?;

        Ok(self.get_script_path(self.use_version_specific_postinstall, &target.postinstall_script, "postinstall"))
    }

    /// Gets the test script path with the given target bounds.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_test_script_path(&self, target_bounds: &TargetBounds) -> Result<String> {
        let target = self.get_target(target_bounds)?;

        Ok(self.get_script_path(self.use_version_specific_test, &target.test_script, "test"))
    }

    /// Gets the uninstall script path with the given target bounds.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_uninstall_script_path(&self, target_bounds: &TargetBounds) -> Result<String> {
        let target = self.get_target(target_bounds)?;

        Ok(self.get_script_path(self.use_version_specific_uninstall, &target.uninstall_script, "uninstall"))
    }

    /// Gets the script arguments for the given target.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_script_args(&self, target_bounds: &TargetBounds) -> Result<HashMap<&str, &str>> {
        let target = self.get_target(target_bounds)?;

        Ok(self.script_args.iter().chain(target.script_args.iter()).map(|(key, value)| (key.as_str(), value.as_str())).collect())
    }

    /// Gets the external test files for the given target.
    /// Returns a `RepositoryError::TargetError` if the target cannot be found.
    pub fn get_external_test_files(&self, target_bounds: &TargetBounds) -> Result<HashSet<&str>> {
        let target = self.get_target(target_bounds)?;

        Ok(self.external_test_files.iter().chain(target.external_test_files.iter()).map(|x| x.as_str()).collect())
    }

    /// Gets the number of revisions of the current package version metadata.
    pub fn get_revision_count(&self) -> u64 {
        self.revisions.len() as u64
    }

    /// Checks if there are conflicts in the package version metadata
    pub fn has_conflicts(&self) -> bool {
        // Check if a global dependency is also specified as target specific dependency
        for dependency in &self.dependencies {
            for target in self.targets.values() {
                for target_dependency in &target.dependencies {
                    if dependency.get_name() == target_dependency.get_name() {
                        return true;
                    }
                }
            }
        }

        // Check if a global build dependency is also specified as target specific build dependency
        for dependency in &self.build_dependencies {
            for target in self.targets.values() {
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
            for target in self.targets.values() {
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

    fn is_default_skip_symlinking(val: &bool) -> bool {
        *val == Self::default_skip_symlinking()
    }

    fn default_use_version_specific() -> bool {
        false
    }

    fn is_default_use_version_specific(val: &bool) -> bool {
        *val == Self::default_use_version_specific()
    }
}
