// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::types::{Dependency, PackageName, Version, VersionIntervals},
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        types::TargetBounds,
    },
};

/// Represents the package metadata, containing package information.
#[derive(Deserialize, Debug)]
pub struct PackageMeta {
    pub name: PackageName,
    pub description: String,
    pub homepage: Option<String>,
    pub versions: Vec<Version>,
    pub supported_versions: HashMap<TargetBounds, VersionIntervals>,
}

impl PackageMeta {
    /// Gets the latest supported version for the current target.
    pub fn get_latest_version(&self, target: &Target) -> Result<&Version> {
        let target = match TargetBounds::get_best_target(&target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;

        // The versions vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<&Version> = None;
        for version in &self.versions {
            // Continue if the version is not supported by the current target
            if !supported.covers(version) {
                continue;
            }

            current_highest = match current_highest {
                Some(highest) if highest < version => Some(version),
                None => Some(version),
                _ => continue,
            };
        }

        Ok(current_highest.ok_or(RepositoryError::SupportError(self.name.to_string()))?)
    }

    /// Gets the latest supported version for the current target which also satisfies the given dependency.
    pub fn get_latest_dependency_version(&self, dependency: &Dependency, target: &Target) -> Result<&Version> {
        let target = match TargetBounds::get_best_target(&target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;

        // The versions vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<&Version> = None;
        for version in &self.versions {
            // Continue if the dependency is not satisfied or if the version is not supported by the current target
            if !dependency.satisfied(&self.name, version) || !supported.covers(version) {
                continue;
            }

            current_highest = match current_highest {
                Some(highest) if highest < version => Some(version),
                None => Some(version),
                _ => continue,
            };
        }

        Ok(current_highest.ok_or(RepositoryError::DependencySupportError(dependency.to_string()))?)
    }
}
