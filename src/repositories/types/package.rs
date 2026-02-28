use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::types::{Dependency, Version},
    repositories::error::{RepositoryError, Result},
};

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

    pub fn get_latest_dependency_version(&self, dependency: &Dependency) -> Result<Version> {
        // The supported vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<Version> = None;
        for version in &self.versions {
            if !dependency.satisfied(&self.name, Some(&version)) {
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
