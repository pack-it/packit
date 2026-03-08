use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de};

use crate::installer::types::{PackageName, Version, VersionBounds, version_intervals::VersionIntervals};

#[derive(Debug, Clone)]
pub struct Dependency {
    name: PackageName,
    version_intervals: VersionIntervals,
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

        // Remove @ character from version number
        let version = version.strip_prefix("@").unwrap_or("");

        let version_intervals = VersionIntervals::from_str(version).map_err(de::Error::custom)?;

        Ok(Self {
            name: PackageName::from_str(name).map_err(de::Error::custom)?,
            version_intervals,
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
        if self.version_intervals.is_empty() {
            write!(f, "{}", self.name)?;
            return Ok(());
        }

        let mut string_version = String::new();
        for range in self.version_intervals.get_version_bounds() {
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
    pub fn get_name(&self) -> &PackageName {
        &self.name
    }

    pub fn satisfied(&self, name: &PackageName, version: &Version) -> bool {
        if self.name != *name {
            return false;
        }

        self.version_intervals.covers(version)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use super::*;

    pub fn create_dependency(name: &str, version_intervals: &str) -> Dependency {
        Dependency {
            name: PackageName::from_str(name).expect("Expected valid package name"),
            version_intervals: VersionIntervals::from_str(version_intervals).expect("Expected correct version intervals."),
        }
    }

    #[test]
    fn satisfied_range() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let dependency = create_dependency("test", "3.4.1-3.4.8");

        assert!(dependency.satisfied(&package_name, &Version::from_str("3.4.8").expect("Expected Version.")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("3.4.0").expect("Expected version")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("3.4.9").expect("Expected version")));
    }

    #[test]
    fn satisfied_lower() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let dependency = create_dependency("test", "<3.4.1");

        assert!(dependency.satisfied(&package_name, &Version::from_str("3.4.0").expect("Expected Version.")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("3.4.1").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_lower_equals() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let dependency = create_dependency("test", "<=3.4.1");

        assert!(dependency.satisfied(&package_name, &Version::from_str("3.4.1").expect("Expected Version.")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("3.4.2").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_higher() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let dependency = create_dependency("test", ">3.4.1");

        assert!(dependency.satisfied(&package_name, &Version::from_str("3.4.2").expect("Expected Version.")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("3.4.1").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_higher_equals() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let dependency = create_dependency("test", ">=3.4.1");

        assert!(dependency.satisfied(&package_name, &Version::from_str("3.4.1").expect("Expected Version.")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("3.4.0").expect("Expected Version.")));
    }

    #[test]
    fn satisfied_equals() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let dependency = create_dependency("test", "3.4.1");

        assert!(dependency.satisfied(&package_name, &Version::from_str("3.4.1").expect("Expected Version.")));
        assert!(!dependency.satisfied(&package_name, &Version::from_str("5").expect("Expected Version.")));
    }
}
