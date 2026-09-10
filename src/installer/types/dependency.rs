// SPDX-License-Identifier: GPL-3.0-only
use serde::{
    Deserialize, Serialize,
    de::{self, Error},
};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use crate::installer::types::{PackageName, Version, version_intervals::VersionIntervals};

/// Errors that occur when creating or using the dependencies.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub enum DependencyError {
    #[error("Expected a version, because '@' was used")]
    ExpectedVersion,
}

/// Holds a dependency name and its allowed versions.
#[cfg_attr(test, derive(PartialEq))]
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

        let (name, version) = match string.split_once('@') {
            Some(value) if value.1.is_empty() => return Err(Error::custom(DependencyError::ExpectedVersion)),
            Some(value) => value,
            None => (string.as_str(), ""),
        };

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

        write!(f, "{}@{}", self.name, self.version_intervals)?;
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

    use serde::de::{
        IntoDeserializer,
        value::{Error, StrDeserializer},
    };

    use crate::installer::types::{
        package_name::tests::create_package_name, version::tests::create_version, version_intervals_test::create_version_intervals,
    };

    use super::*;

    /// This is a helper method which creates a `Dependency` from a name and version_intervals which are assumed to be correct.
    pub fn create_dependency(name: &str, version_intervals: &str) -> Dependency {
        Dependency {
            name: create_package_name(name),
            version_intervals: VersionIntervals::from_str(version_intervals).expect("Expected correct version intervals"),
        }
    }

    #[test]
    fn deserialize() {
        let input: StrDeserializer<'_, Error> = "test".into_deserializer();
        assert_eq!(
            Dependency::deserialize(input),
            Ok(Dependency {
                name: create_package_name("test"),
                version_intervals: create_version_intervals("")
            })
        );

        let input: StrDeserializer<'_, Error> = "test@1.1".into_deserializer();
        assert_eq!(
            Dependency::deserialize(input),
            Ok(Dependency {
                name: create_package_name("test"),
                version_intervals: create_version_intervals("1.1")
            })
        );
    }

    #[test]
    fn deserialize_missing_version() {
        let input: StrDeserializer<'_, Error> = "test@".into_deserializer();
        assert_eq!(
            Dependency::deserialize(input),
            Err(DependencyError::ExpectedVersion).map_err(de::Error::custom)
        );

        let input: StrDeserializer<'_, Error> = "@".into_deserializer();
        assert_eq!(
            Dependency::deserialize(input),
            Err(DependencyError::ExpectedVersion).map_err(de::Error::custom)
        );
    }

    #[test]
    fn satisfied_range() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "3.4.1-3.4.8");

        assert!(dependency.satisfied(&package_name, &create_version("3.4.7")));
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.0")));
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.8")));
    }

    #[test]
    fn satisfied_lower() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "<3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version("3.4.0")));
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.1")));
    }

    #[test]
    fn satisfied_lower_equals() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "<=3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version("3.4.1")));
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.2")));
    }

    #[test]
    fn satisfied_higher() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", ">3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version("3.4.2")));
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.1")));
    }

    #[test]
    fn satisfied_higher_equals() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", ">=3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version("3.4.1")));
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.0")));
    }

    #[test]
    fn satisfied_equals() {
        let package_name = create_package_name("test");
        let dependency = create_dependency("test", "3.4.1");

        assert!(dependency.satisfied(&package_name, &create_version("3.4.1")));
        assert!(!dependency.satisfied(&package_name, &create_version("5")));
    }

    #[test]
    fn satisfied_wrong_name() {
        let package_name = create_package_name("wrong_name");
        let dependency = create_dependency("test", "3.4.1");
        assert!(!dependency.satisfied(&package_name, &create_version("3.4.1")));
    }

    #[test]
    fn format() {
        let dependency = create_dependency("test", "3.4.1");
        assert_eq!(dependency.to_string(), "test@3.4.1");

        let dependency = create_dependency("test", "");
        assert_eq!(dependency.to_string(), "test");

        let dependency = create_dependency("test", "3.4.0-3.4.1");
        assert_eq!(dependency.to_string(), "test@3.4.0-3.4.1");
    }
}
