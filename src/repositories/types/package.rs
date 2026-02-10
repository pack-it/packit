use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    installer::types::Version,
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
}
