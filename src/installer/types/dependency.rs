use std::fmt::Display;

use serde::{de, Deserialize, Serialize};
use thiserror::Error;

use crate::installer::types::{Version, VersionBounds, VersionError};

#[derive(Error, Debug)]
pub enum DependencyParserError {
    #[error("Caused by: Cannot parse version number")]
    VersionNumberError(#[from] VersionError),

    #[error("No bounds specified while requested with '@'")]
    EmptyBoundsError,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    name: String,
    version_ranges: Vec<VersionBounds>,
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        let index = string.chars().position(|c| c == '@');

        let (name, version) = match index {
            Some(index) => string.split_at(index),
            None => {
                return Ok(Self {
                    name: string.to_string(),
                    version_ranges: vec![],
                })
            },
        };

        // Remove @ character from version number
        let version = version.strip_prefix("@").unwrap_or("");

        let version_ranges = VersionBounds::from_str_ranges(version).map_err(de::Error::custom)?;

        Ok(Self {
            name: name.to_string(),
            version_ranges,
        })
    }
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Return only the name if the version isn't specified
        if self.version_ranges.is_empty() {
            write!(f, "{}", self.name)?;
            return Ok(());
        }

        let mut string_version = String::new();
        for range in &self.version_ranges {
            if !string_version.is_empty() {
                string_version.push('|');
            }

            match range {
                VersionBounds::Range(lower, upper) => string_version.push_str(&format!("{}-{}", lower.to_string(), upper.to_string())),
                VersionBounds::Lower(version) => string_version.push_str(&format!("<{}", version.to_string())),
                VersionBounds::LowerEqual(version) => string_version.push_str(&format!("<={}", version.to_string())),
                VersionBounds::Higher(version) => string_version.push_str(&format!(">{}", version.to_string())),
                VersionBounds::HigherEqual(version) => string_version.push_str(&format!(">={}", version.to_string())),
                VersionBounds::Equal(version) => string_version.push_str(&format!("={}", version.to_string())),
            }
        }

        write!(f, "{}@{}", &self.name, &string_version)?;
        Ok(())
    }
}

impl Dependency {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn satisfied(&self, name: &str, version: Option<&Version>) -> bool {
        if self.name != name {
            return false;
        }

        let version = match version {
            Some(version) => version,
            None => return true,
        };

        if self.version_ranges.is_empty() {
            return true;
        }

        for range in &self.version_ranges {
            return match range {
                VersionBounds::Range(lower, upper) if lower <= version && upper >= version => true,
                VersionBounds::Lower(lower) if version < lower => true,
                VersionBounds::LowerEqual(lower) if version <= lower => true,
                VersionBounds::Higher(higher) if version > higher => true,
                VersionBounds::HigherEqual(higher) if version >= higher => true,
                VersionBounds::Equal(equal) if version == equal => true,
                _ => continue,
            };
        }

        false
    }
}
