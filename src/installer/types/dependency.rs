// SPDX-License-Identifier: GPL-3.0-only
use serde::{Deserialize, Serialize, de};
use std::{fmt::Display, str::FromStr};

use crate::installer::types::{PackageName, Version, VersionBounds, version_intervals::VersionIntervals};

/// Holds a dependency name and its allowed versions.
#[derive(Debug, Clone)]
pub struct Dependency {
    name: PackageName,
    version_intervals: VersionIntervals,
}

impl<'de> Deserialize<'de> for Dependency {
    /// Deserializes a string into a Dependency.
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
    /// Serializes a Dependency into a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Display for Dependency {
    /// Formats the current dependency in the following style: `<name>[@version-intervals]`.
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
                VersionBounds::Range(low, high) => string_version.push_str(&format!("{low}-{high}")),
                VersionBounds::IncludingRange(low, high) => string_version.push_str(&format!("{low}-={high}")),
                VersionBounds::Lower(version) => string_version.push_str(&format!("<{version}")),
                VersionBounds::LowerEqual(version) => string_version.push_str(&format!("<={version}")),
                VersionBounds::Higher(version) => string_version.push_str(&format!(">{version}")),
                VersionBounds::HigherEqual(version) => string_version.push_str(&format!(">={version}")),
                VersionBounds::Equal(version) => string_version.push_str(&format!("={version}")),
            }
        }

        write!(f, "{}@{}", self.name, string_version)?;
        Ok(())
    }
}

impl Dependency {
    /// Gets the dependency name.
    pub fn get_name(&self) -> &PackageName {
        &self.name
    }

    /// Checks if a given name and version satisfy the current dependency.
    /// Returns true if it does, false otherwise.
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

    use crate::installer::types::package_name::tests::create_package_name;
    use crate::installer::types::version::tests::create_version;

    use super::*;

    /// This is a helper method which creates a `Dependency` from a name and version_intervals which are assumed to be correct.
    pub fn create_dependency(name: &str, version_intervals: &str) -> Dependency {
        Dependency {
            name: create_package_name(name),
            version_intervals: VersionIntervals::from_str(version_intervals).expect("Expected correct version intervals"),
        }
    }

    #[test]
    fn satisfied_range() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "3.4.1-3.4.8");

        assert!(dependency.satisfied(&package_name, &create_version(&[3, 4, 7])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[3, 4, 0])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[3, 4, 8])));
    }

    #[test]
    fn satisfied_lower() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "<3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version(&[3, 4, 0])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[3, 4, 1])));
    }

    #[test]
    fn satisfied_lower_equals() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "<=3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version(&[3, 4, 1])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[3, 4, 2])));
    }

    #[test]
    fn satisfied_higher() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", ">3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version(&[3, 4, 2])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[3, 4, 1])));
    }

    #[test]
    fn satisfied_higher_equals() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", ">=3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version(&[3, 4, 1])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[3, 4, 0])));
    }

    #[test]
    fn satisfied_equals() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version(&[3, 4, 1])));
        assert!(!dependency.satisfied(&package_name, &create_version(&[5])));
    }
}
