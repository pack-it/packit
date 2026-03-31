use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Wrapper to differentiate between different License types in metadata files.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Licenses {
    None,
    Single(String),
    Any {
        any: Vec<String>,
    },
    All {
        all: Vec<String>,
    },
}

impl Licenses {
    /// Returns true if the License is `None`.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl Display for Licenses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Licenses::None => write!(f, "None"),
            Licenses::Single(license) => write!(f, "{license}"),
            Licenses::Any { any } => write!(f, "any of: {}", any.join(", ")),
            Licenses::All { all } => write!(f, "all of: {}", all.join(", ")),
        }
    }
}

impl Default for Licenses {
    fn default() -> Self {
        Self::None
    }
}
