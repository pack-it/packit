use serde::Deserialize;

use crate::repositories::types::Licenses;

/// Represents the repository metadata, containing repository information.
#[derive(Deserialize, Debug)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,

    #[serde(skip_serializing_if = "Licenses::is_unknown", default)]
    pub license: Licenses,

    pub prebuilds_url: Option<String>,
    pub prebuilds_provider: Option<String>,
}
