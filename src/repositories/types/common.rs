// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashMap, fmt::Display, ops::Not};

use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, de};

use crate::repositories::types::Checksum;

/// Represents a script identifier, holding the scripts name and a bool which specifies
/// if the script should be version specific.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    NameOnly(String),
    Expanded {
        name: String,
        version_specific: bool,
    },
}

/// Represents a source, holding a URL and mirror URLs to the source code of a package.
/// Also has a checksum to check the validity of the received source code.
#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub url: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mirrors: Vec<String>,
    pub checksum: Checksum,
    pub size: FileSize,

    #[serde(default, skip_serializing_if = "<&bool>::not")]
    pub skip_unpack: bool,
    pub apply_patches_in: Option<String>,

    #[serde(default, deserialize_with = "Source::deserialize_patches", skip_serializing_if = "HashMap::is_empty")]
    pub patches: HashMap<u32, Patch>,
}

/// Wrapper to differentiate between Single and Named sources in the metadata files.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Sources {
    Single(Source),
    Named(HashMap<String, Source>),
}

/// Represents a patch to a source file, holding a URL, mirror URLs and a checksum to check validity.
#[derive(Serialize, Deserialize, Debug)]
pub struct Patch {
    pub url: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mirrors: Vec<String>,
    pub checksum: Checksum,
    pub apply_in: Option<String>,
}

/// Represents the size of a file in bytes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSize(pub u32);

/// Represents a date in the metadata.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(chrono::NaiveDate);

/// Represents information about a package deprecation and disabling.
#[derive(Serialize, Deserialize, Debug)]
pub struct DeprecationInfo {
    pub deprecated_from: Date,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_from: Option<Date>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Source {
    /// Gets all patches of the source, sorted by id.
    pub fn get_sorted_patches(&self) -> Vec<(u32, &Patch)> {
        let mut vec: Vec<(u32, &Patch)> = self.patches.iter().map(|(key, value)| (*key, value)).collect();
        vec.sort_by_key(|(key, _)| *key);
        vec
    }

    /// Custom deserializer to deserialize integer keys correctly
    fn deserialize_patches<'de, D>(deserializer: D) -> Result<HashMap<u32, Patch>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, Patch>::deserialize(deserializer)?;

        map.into_iter()
            .map(|(key, value)| match key.parse() {
                Ok(key) => Ok((key, value)),
                Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(&key), &"a non-negative integer")),
            })
            .collect()
    }
}

// Custom deserialize implementation of source to differentiate between single and named sources.
impl<'de> Deserialize<'de> for Sources {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: toml::Value = Deserialize::deserialize(deserializer)?;

        // If the toml contains an url and a checksum, we assume it is a single source
        if value.get("url").is_some() && value.get("checksum").is_some() {
            let single = Source::deserialize(value).map_err(de::Error::custom)?;

            return Ok(Sources::Single(single));
        }

        let named = HashMap::deserialize(value).map_err(de::Error::custom)?;
        Ok(Sources::Named(named))
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut unit = 0;
        let mut value = self.0 as f64;

        // Adjust unit and value to order of size
        while unit < Self::UNITS.len() && value > 1750.0 {
            unit += 1;
            value /= 1024.0;
        }

        // Show 2 decimal places and the correct unit
        write!(f, "{:.2} {}", value, Self::UNITS[unit])
    }
}

impl FileSize {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl Date {
    /// Gets the current date.
    pub fn now() -> Self {
        Self(Utc::now().date_naive())
    }
}
