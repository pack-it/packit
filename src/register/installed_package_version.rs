// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{config::Repository, installer::types::PackageId, register::metadata::LocalMetaHandler};

/// Represents a specific package version which is installed on the system.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledPackageVersion {
    pub package_id: PackageId,

    #[serde(default = "Repository::default_repository_provider")]
    #[serde(skip_serializing_if = "is_repository_provider_default")]
    pub metadata_repository_provider: String,
    pub metadata_repository_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds_repository_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds_repository_provider: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub dependencies: HashSet<PackageId>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub dependents: HashSet<PackageId>,

    pub install_path: PathBuf,

    #[serde(default)]
    pub revisions: Vec<String>,

    // The default on the `last_metadata_refresh` and `last_metadata_change` is required to ensure backwards compatibility
    #[serde(default)]
    pub last_metadata_refresh: DateTime<Utc>,

    #[serde(default)]
    pub last_metadata_change: DateTime<Utc>,
}

fn is_repository_provider_default(value: &String) -> bool {
    *value == Repository::default_repository_provider()
}

impl InstalledPackageVersion {
    /// Gets the local metadata handler for the installed package version.
    pub fn get_local_metadata(&self) -> LocalMetaHandler<'_> {
        LocalMetaHandler::new(&self.package_id, &self.install_path)
    }

    // Updates the `last_metadata_refresh` and the `last_metadata_change` based on the `updated` paramter.
    pub fn update_metadata_refresh(&mut self, updated_metadata: bool) {
        let now = Utc::now();

        self.last_metadata_refresh = now;
        if updated_metadata {
            self.last_metadata_change = now;
        }
    }
}
