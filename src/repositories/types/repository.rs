// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{installer::types::Version, repositories::types::Licenses};

/// Represents the repository metadata, containing repository information.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,
    pub required_packit_version: Version,

    #[serde(default, skip_serializing_if = "Licenses::is_unknown")]
    pub license: Licenses,

    /// Specifies a suggestion of a prebuild repository to use with this metadata repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds_url: Option<String>,

    /// Specifies a suggestion of a prebuild repository to use with this metadata repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds_provider: Option<String>,

    /// A set of compatible repositories
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub compatible_repositories: HashSet<String>,
}
