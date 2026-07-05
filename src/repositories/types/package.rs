// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    installer::types::{PackageName, Version, VersionIntervals},
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        types::{DeprecationInfo, TargetBounds},
    },
};

/// Represents the package metadata, containing package information.
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMeta {
    pub name: PackageName,
    pub description: String,
    pub homepage: Option<String>,
    pub versions: Vec<Version>,
    pub required_packit_version: Option<Version>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts_with: Vec<PackageName>,
    pub supported_versions: HashMap<TargetBounds, VersionIntervals>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationInfo>,
}

impl PackageMeta {
    /// Gets a list of all supported versions for the given target.
    pub fn get_supported_versions(&self, target: &Target) -> Result<Vec<&Version>> {
        let target = match TargetBounds::get_best_target(target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;

        // Get all supported versions and sort them
        let mut versions = Vec::new();
        for version in &self.versions {
            // Continue if the version is not supported by the current target
            if !supported.covers(version) {
                continue;
            }

            versions.push(version);
        }
        versions.sort();

        // Return a support error if no supported versions are found
        if versions.is_empty() {
            return Err(RepositoryError::SupportError(self.name.to_string()));
        }

        Ok(versions)
    }
}
