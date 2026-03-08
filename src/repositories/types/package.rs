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
    pub supported_versions: HashMap<TargetBounds, VersionIntervals>,
}

impl PackageMeta {
    pub fn get_latest_version(&self, target: &Target) -> Result<&Version> {
        let target = match TargetBounds::get_best_target(&target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;
        Ok(self.latest_version_impl(|v| !supported.covers(v))?.ok_or(RepositoryError::EmptyIntervals)?)
    }

    pub fn get_latest_dependency_version(&self, dependency: &Dependency, target: &Target) -> Result<&Version> {
        let target = match TargetBounds::get_best_target(&target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;
        Ok(self
            .latest_version_impl(|v| !dependency.satisfied(&self.name, v) || !supported.covers(v))?
            .ok_or(RepositoryError::SupportError(dependency.to_string()))?)
    }

    /// A generic method to get the latest version of the current package.
    /// If any of the checks are true for the current version we continue for that version.
    fn latest_version_impl<F>(&self, checks: F) -> Result<Option<&Version>>
    where
        F: Fn(&Version) -> bool,
    {
        // The versions vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<&Version> = None;
        for version in &self.versions {
            // Continue if any of the checks are true
            if checks(version) {
                continue;
            }

            current_highest = match current_highest {
                Some(highest) if highest < version => Some(version),
                None => Some(version),
                _ => continue,
            };
        }

        Ok(current_highest)
    }
}
