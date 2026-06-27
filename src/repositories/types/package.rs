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
    pub supported_versions: HashMap<TargetBounds, VersionIntervals>,
    pub deprecation: Option<DeprecationInfo>,
}

impl PackageMeta {
    pub fn get_supported_versions(&self, target: &Target) -> Result<Vec<&Version>> {
        let target = match TargetBounds::get_best_target(target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;

        // The versions vec isn't necessary in order, so we need to keep track of the current highest version
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
