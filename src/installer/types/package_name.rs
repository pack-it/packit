// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, ops::Deref, path::Path, str::FromStr, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Serialize, de};
use thiserror::Error;

const VALID_PACKAGE_NAME: &str = r"^[a-zA-Z0-9\-_]+$";
const PACKAGE_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(VALID_PACKAGE_NAME).expect("Expected valid regex"));

/// Errors that occur when creating or parsing the package name.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub enum PackageNameError {
    #[error("Package name cannot be empty and can only contain characters: 'a-z', 'A-Z', '0-9', '-' and '_'")]
    InvalidPackageName,
}

/// Represents the name of a package.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackageName(String);

impl PackageName {
    /// Get the prefix from the package name. The package name regex assures that one character is always present.
    pub fn get_prefix(&self) -> char {
        self.0.chars().next().expect("Expected first char, based on regex")
    }

    /// Gets the package name of Packit itself.
    pub fn packit() -> Self {
        Self("packit".to_string())
    }
}

impl<'de> Deserialize<'de> for PackageName {
    /// Deserializes a string into a `PackageName`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;

        Self::from_str(&string).map_err(de::Error::custom)
    }
}

impl Serialize for PackageName {
    /// Serializes the `PackageName` into a string. Note that this doesn't check its validity, it assumes
    /// that the `PackageName` validity is always checked upon creation.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

impl Display for PackageName {
    /// Formats a `PackageName` into the following format: <name>.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl FromStr for PackageName {
    type Err = PackageNameError;

    /// Parses a string into a `PackageName`.
    /// Could return a `PackageNameError::InvalidPackageName` error.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if !PACKAGE_NAME_REGEX.is_match(string) {
            return Err(PackageNameError::InvalidPackageName);
        }

        Ok(Self(string.to_string()))
    }
}

/// Implements `Deref` for `PackageName`.
impl Deref for PackageName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Implements `AsRef<Path>` for `PackageName`.
impl AsRef<Path> for PackageName {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use super::*;

    /// This is a helper method which creates a `PackageName` from a name which is assumed to be correct.
    pub fn create_package_name(name: &str) -> PackageName {
        PackageName(name.to_string())
    }

    #[test]
    fn valid_from_str() {
        let name = &"_Test-123";
        assert_eq!(PackageName::from_str(name), Ok(PackageName(name.to_string())));
    }

    #[test]
    fn from_str_no_input() {
        assert_eq!(PackageName::from_str(""), Err(PackageNameError::InvalidPackageName));
    }

    #[test]
    fn from_str_illegal_chars() {
        let illegal_chars = " ./\\!@#$%^&*():;'\"<>[]{}?|~`±§=+\u{1234}";
        for name in illegal_chars.chars() {
            assert_eq!(
                PackageName::from_str(&name.to_string()),
                Err(PackageNameError::InvalidPackageName),
                "expected {name:?} to be invalid"
            );
        }
    }

    #[test]
    fn format() {
        let package_name = PackageName("_Test-123".to_string());
        assert_eq!(package_name.to_string(), "_Test-123");
    }

    #[test]
    fn get_prefix() {
        let package_name = PackageName("_Test-123".to_string());
        assert_eq!(package_name.get_prefix(), '_');
    }

    #[test]
    fn get_packit_name() {
        assert_eq!(PackageName::packit(), PackageName("packit".to_string()));
    }
}
