use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::types::{Dependency, Version, VersionIntervals},
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        types::target_bounds::TargetBounds,
    },
};

/// Represents the package metadata, containing package information.
/// TODO: Validate name with PackageId rules
#[derive(Deserialize, Debug)]
pub struct PackageMeta {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub versions: Vec<Version>,
    pub supported_versions: HashMap<TargetBounds, VersionIntervals>, // TODO: Maybe make sure these intervals don't use higher than bounds
}

impl PackageMeta {
    pub fn get_latest_version(&self, target: &Target) -> Result<&Version> {
        let target = match TargetBounds::get_best_target(&target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let current_versions = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;
        if let Some(last_bound) = current_versions.get_version_bounds().last() {
            return Ok(last_bound.get_upperbound().ok_or(RepositoryError::InvalidSupportIntervals)?);
        }

        Err(RepositoryError::InvalidSupportIntervals)
    }

    pub fn get_latest_dependency_version(&self, dependency: &Dependency, target: &Target) -> Result<Version> {
        let target = match TargetBounds::get_best_target(&target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        // The supported vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<Version> = None;
        for version in &self.versions {
            // Continue if the dependency is not satisfied
            if !dependency.satisfied(&self.name, version) {
                continue;
            }

            // Continue if the version is not supported for the current target
            if !self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?.covers(version) {
                continue;
            }

            current_highest = match current_highest {
                Some(highest) if highest < *version => Some(version.clone()),
                None => Some(version.clone()),
                _ => continue,
            };
        }

        Ok(current_highest.ok_or(RepositoryError::SupportError(dependency.to_string()))?)
    }
}
