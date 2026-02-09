use std::{fmt::Display, str::FromStr};

use serde::{de, Deserialize, Serialize};
use thiserror::Error;

use crate::installer::types::{Version, VersionError};

/// Errors that occur when creating or using the package id.
#[derive(Error, Debug, PartialEq)]
pub enum PackageIdError {
    #[error("No name found, package id requires a name.")]
    NoNameError,

    #[error("Couldn't parse package id, because of an invalid version.")]
    VersionError(#[from] VersionError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub version: Version,
}

impl PackageId {
    pub fn new(name: &str, version: &Version) -> Self {
        Self {
            name: name.to_string(),
            version: version.clone(),
        }
    }
}

impl<'de> Deserialize<'de> for PackageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(&string).map_err(de::Error::custom)?)
    }
}

impl Serialize for PackageId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", &self.name, &self.version)?;
        Ok(())
    }
}

impl FromStr for PackageId {
    type Err = PackageIdError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let index = string.chars().position(|c| c == '@');

        let (name, version) = match index {
            Some(index) => string.split_at(index),
            None => return Err(VersionError::NoneError)?,
        };

        // Remove @ character from version number before converting to Version
        let version = Version::from_str(&version[1..])?;

        // Name must have some value
        if name.is_empty() {
            return Err(PackageIdError::NoNameError);
        }

        Ok(Self {
            name: name.to_string(),
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let correct_version = PackageId::new("Test", &Version::from_str("3.4.1").expect("Expected Version."));

        match PackageId::from_str("Test@3.4.1") {
            Ok(id) => assert_eq!(id, correct_version),
            Err(e) => panic!("Expected Ok(PackageId(name: 'Test', version: Version(..))), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_no_version() {
        assert_eq!(
            PackageId::from_str("Test"),
            Err(PackageIdError::VersionError(VersionError::NoneError))
        );
    }

    #[test]
    fn from_str_no_name() {
        assert_eq!(PackageId::from_str("@3.4.1"), Err(PackageIdError::NoNameError));
    }

    #[test]
    fn valid_format() {
        let correct_version = PackageId::new("Test", &Version::from_str("3.4.1").expect("Expected Version."));

        assert_eq!(correct_version.to_string(), "Test@3.4.1");
    }
}
