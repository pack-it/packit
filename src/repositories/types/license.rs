// SPDX-License-Identifier: GPL-3.0-only
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Wrapper to differentiate between different License types in metadata files.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum Licenses {
    #[default]
    Unknown,
    Single(String),
    Any {
        any: Vec<String>,
    },
    All {
        all: Vec<String>,
    },
}

impl Licenses {
    /// Returns true if the License is `Unknown`.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl Display for Licenses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Licenses::Unknown => write!(f, "Unknown"),
            Licenses::Single(license) => write!(f, "{license}"),
            Licenses::Any { any } => write!(f, "any of: {}", any.join(", ")),
            Licenses::All { all } => write!(f, "all of: {}", all.join(", ")),
        }
    }
}
