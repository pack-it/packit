// SPDX-License-Identifier: GPL-3.0-only
use serde::{Deserialize, Serialize};

use crate::repositories::types::Licenses;

/// Represents the repository metadata, containing repository information.
#[derive(Deserialize, Serialize, Debug)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,

    #[serde(skip_serializing_if = "Licenses::is_unknown", default)]
    pub license: Licenses,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds_provider: Option<String>,
}
