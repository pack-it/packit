use std::fmt::Display;

use serde::{Deserialize, Serialize, de};
use thiserror::Error;

use crate::installer::types::{PackageId, Version, VersionBounds, VersionError};

#[derive(Error, Debug, PartialEq)]
pub enum DependencyParserError {
    #[error("Cannot parse version number")]
    VersionNumberError(#[from] VersionError),

    #[error("Invalid dependency name, a package name cannot be empty and can only contain characters: 'a-z', 'A-Z', '0-9', '-' and '_'")]
    InvalidDependencyName,
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
            None => (string.as_str(), ""),
        };

        // Check for name validity
        if !PackageId::is_valid_name(name) {
            return Err(de::Error::custom(DependencyParserError::InvalidDependencyName));
        }

        // Remove @ character from version number
        let version = version.strip_prefix("@").unwrap_or("");

        // TODO: Check for impossible requirements, like: >3.1|<3.1
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

    pub fn satisfied(&self, name: &str, version: &Version) -> bool {
        if self.name != name {
            return false;
        }

        if self.version_ranges.is_empty() {
            return true;
        }

        for range in &self.version_ranges {
            if range.covers(&version) {
                return true;
            }
        }

        false
    }

    pub fn to_package_id(&self, version: Version) -> PackageId {
        PackageId::new(&self.name, version).expect("Expected valid name from dependency.")
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use super::*;

    pub fn create_dependency(name: &str, version_ranges: &str) -> Dependency {
        Dependency {
            name: name.into(),
            version_ranges: VersionBounds::from_str_ranges(version_ranges).expect("Expected VersionBounds"),
        }
    }

    #[test]
    fn satisfied_range() {
        let dependency = create_dependency("Test", "3.4.1-3.4.8");

        assert!(dependency.satisfied("Test", &Version::from_str("3.4.8").expect("Expected Version.")));
        assert!(!dependency.satisfied("Test", &Version::from_str("3.4.0").expect("Expected version")));
        assert!(!dependency.satisfied("Test", &Version::from_str("3.4.9").expect("Expected version")));
    }

    #[test]
    fn satisfied_lower() {
        let dependency = create_dependency("Test", "<3.4.1");

        assert!(dependency.satisfied("Test", &Version::from_str("3.4.0").expect("Expected Version.")));
        assert!(!dependency.satisfied("Test", &Version::from_str("3.4.1").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_lower_equals() {
        let dependency = create_dependency("Test", "<=3.4.1");

        assert!(dependency.satisfied("Test", &Version::from_str("3.4.1").expect("Expected Version.")));
        assert!(!dependency.satisfied("Test", &Version::from_str("3.4.2").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_higher() {
        let dependency = create_dependency("Test", ">3.4.1");

        assert!(dependency.satisfied("Test", &Version::from_str("3.4.2").expect("Expected Version.")));
        assert!(!dependency.satisfied("Test", &Version::from_str("3.4.1").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_higher_equals() {
        let dependency = create_dependency("Test", ">=3.4.1");

        assert!(dependency.satisfied("Test", &Version::from_str("3.4.1").expect("Expected Version.")));
        assert!(!dependency.satisfied("Test", &Version::from_str("3.4.0").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_equals() {
        let dependency = create_dependency("Test", "3.4.1");

        assert!(dependency.satisfied("Test", &Version::from_str("3.4.1").expect("Expected Version.")));
        assert!(!dependency.satisfied("Test", &Version::from_str("5").expect("Expected Version.")));
    }
}
