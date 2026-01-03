use std::{fmt::Display, str::FromStr};

use serde::{de, Deserialize, Serialize};
use thiserror::Error;

use crate::version::{Version, VersionError};

#[derive(Error, Debug)]
pub enum DependencyParserError {
    #[error("Cannot parse version number. {0}")]
    VersionNumberError(#[from] VersionError),

    #[error("No bounds specified while request with '@'.")]
    EmptyBoundsError,
}

#[derive(Debug, Clone)]
enum VersionBounds {
    Range(Version, Version),
    Lower(Version),
    LowerEqual(Version),
    Higher(Version),
    HigherEqual(Version),
    Equal(Version),
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
        let s: &str = de::Deserialize::deserialize(deserializer)?;
        let index = s.chars().position(|c| c == '@');

        let (name, version) = match index {
            Some(index) => s.split_at(index),
            None => {
                return Ok(Self {
                    name: s.to_string(),
                    version_ranges: vec![],
                })
            },
        };

        let version_ranges = parse_ranges(version).map_err(de::Error::custom)?;

        Ok(Self {
            name: name.to_string(),
            version_ranges,
        })
    }
}

fn parse_ranges(ranges: &str) -> Result<Vec<VersionBounds>, DependencyParserError> {
    let ranges = ranges.split('|');
    let mut bounds = Vec::new();

    for range in ranges {
        bounds.push(parse_version_range(range)?);
    }

    // Bounds must have at least one item
    if bounds.is_empty() {
        return Err(DependencyParserError::EmptyBoundsError);
    }

    Ok(bounds)
}

fn parse_version_range(version: &str) -> Result<VersionBounds, DependencyParserError> {
    // Check if the statement is a two sided range
    if let Some(index) = version.chars().position(|c| c == '-') {
        if let Some((lower, upper)) = version.split_at_checked(index) {
            return Ok(VersionBounds::Range(Version::from_str(lower)?, Version::from_str(upper)?));
        }
    }

    if let Some(version) = version.strip_prefix('<') {
        return Ok(VersionBounds::Lower(Version::from_str(version)?));
    }

    if let Some(version) = version.strip_prefix("<=") {
        return Ok(VersionBounds::LowerEqual(Version::from_str(version)?));
    }

    if let Some(version) = version.strip_prefix('>') {
        return Ok(VersionBounds::Higher(Version::from_str(version)?));
    }

    if let Some(version) = version.strip_prefix(">=") {
        return Ok(VersionBounds::HigherEqual(Version::from_str(version)?));
    }

    return Ok(VersionBounds::Equal(Version::from_str(version)?));
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

        write!(f, "{}", self.name.clone() + "@" + &string_version)?;
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

    /// Gets the highest possible dependency version
    pub fn get_highest_version(&self) -> Option<&Version> {
        let mut current_highest = None;
        for range in &self.version_ranges {
            let version = match range {
                VersionBounds::Range(_, upper) => upper,
                VersionBounds::Lower(version) => version,
                VersionBounds::LowerEqual(version) => version,
                VersionBounds::Higher(version) => version,
                VersionBounds::HigherEqual(version) => version,
                VersionBounds::Equal(version) => version,
            };

            current_highest = match current_highest {
                Some(highest) if highest < version => Some(version),
                None => Some(version),
                _ => continue,
            }
        }

        current_highest
    }
}
