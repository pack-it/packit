use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de};
use thiserror::Error;

use crate::installer::types::{PackageName, Version, VersionError, package_name::PackageNameError};

/// Errors that occur when creating or using the package id.
#[derive(Error, Debug)]
pub enum PackageIdError {
    #[error("Invalid package id version")]
    VersionError(#[from] VersionError),

    #[error("Invalid package id name")]
    PackageNameError(#[from] PackageNameError),
}

/// Identifies a package with a name and version.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: PackageName,
    pub version: Version,
}

impl PackageId {
    /// Creates a new `PackageId` from a `PackageName` and `Version`.
    pub fn new(name: PackageName, version: Version) -> Self {
        Self { name, version }
    }
}

impl<'de> Deserialize<'de> for PackageId {
    /// Parses a string into a `PackageId` struct.
    /// Could return errors from the `FromStr` trait implementation.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(&string).map_err(de::Error::custom)?)
    }
}

impl Serialize for PackageId {
    /// Parses a `PackageId` struct into a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Display for PackageId {
    /// Formats the `PackageId` into the following format: <name>@<version>.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", &self.name, &self.version)?;
        Ok(())
    }
}

impl FromStr for PackageId {
    type Err = PackageIdError;

    /// Parses a string into a `PackageId`.
    /// Could return a `PackageIdError`.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let index = string.chars().position(|c| c == '@');

        let (name, version) = match index {
            Some(index) => string.split_at(index),
            None => return Err(VersionError::NoneError)?,
        };

        // Remove @ character from version number before converting to Version
        let version = Version::from_str(&version[1..])?;

        Ok(Self {
            name: PackageName::from_str(name)?,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let version = Version::from_str("3.4.1").expect("Expected Version.");
        let correct_version = PackageId::new(package_name, version);

        match PackageId::from_str("test@3.4.1") {
            Ok(id) => assert_eq!(id, correct_version),
            Err(e) => panic!("Expected Ok(PackageId(name: 'test', version: Version(..))), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_no_version() {
        assert!(matches!(
            PackageId::from_str("test"),
            Err(PackageIdError::VersionError(VersionError::NoneError))
        ));
    }

    #[test]
    fn from_str_no_name() {
        assert!(matches!(
            PackageId::from_str("@3.4.1"),
            Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
        ));
    }

    #[test]
    fn from_str_invalid_chars() {
        let invalid_chars = "!#$%^&*()~:;{}[]<>,.?/|\\\"\'`+=";
        for char in invalid_chars.chars() {
            assert!(matches!(
                PackageId::from_str(format!("{char}@3.4.1").as_str()),
                Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
            ));
        }
    }

    #[test]
    fn valid_format() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let version = Version::from_str("3.4.1").expect("Expected Version.");
        let correct_version = PackageId::new(package_name, version);

        assert_eq!(correct_version.to_string(), "test@3.4.1");
    }
}
