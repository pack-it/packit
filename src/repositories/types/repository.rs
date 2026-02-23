use serde::Deserialize;

/// Represents the repository metadata, containing repository information.
#[derive(Deserialize, Debug)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,

    pub prebuilds_url: Option<String>,
    pub prebuilds_provider: Option<String>,
}
