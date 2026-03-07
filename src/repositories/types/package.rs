use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::types::{Dependency, Version},
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
    pub latest_versions: HashMap<TargetBounds, Version>,
}

impl PackageMeta {
    pub fn get_latest_version(&self, target: &Target) -> Result<&Version> {
        let target = match TargetBounds::get_best_target(&target, self.latest_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        Ok(self.latest_versions.get(target).ok_or(RepositoryError::TargetError)?)
    }

    // TODO: Bug, this needs to adjust for the platform (so it should compare with get_latest_version)
    pub fn get_latest_dependency_version(&self, dependency: &Dependency) -> Result<Version> {
        // The supported vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<Version> = None;
        for version in &self.versions {
            if !dependency.satisfied(&self.name, &version) {
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
