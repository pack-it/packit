// SPDX-License-Identifier: GPL-3.0-only
use std::{
    cmp::{Ordering, max},
    fmt::Display,
    hash::Hash,
    num::ParseIntError,
    str::FromStr,
};

use serde::{Deserialize, Serialize, de};
use thiserror::Error;

use crate::installer::types::version_number::VersionNumber;

/// Errors that occur when parsing version related structs.
#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Version is none or part of version is none.")]
    NoneError,

    #[error("Version number contains a character which is not a digit or a dot.")]
    IllegalCharacterError,

    #[error("Invalid version interval, an interval must be ordered and not overlapping.")]
    InvalidInterval,

    #[error("Couldn't parse version number")]
    ParseError(#[from] ParseIntError),
}

/// Represents a version.
#[derive(Debug, Eq, Clone)]
pub struct Version {
    numbers: Vec<VersionNumber>,
}

impl<'de> Deserialize<'de> for Version {
    /// Deserializes a string into a `Version`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;

        Version::from_str(&string).map_err(de::Error::custom)
    }
}

impl Serialize for Version {
    /// Serializes a `Version` into a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Ord for Version {
    /// Compares this version to another version and returns an `Ordering`.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let iterations = max(self.numbers.len(), other.numbers.len());
        for i in 0..iterations {
            let num = match self.numbers.get(i) {
                Some(num) => num,
                None => &VersionNumber::from(0),
            };

            let other_num = match other.numbers.get(i) {
                Some(num) => num,
                None => &VersionNumber::from(0),
            };

            if num == other_num {
                continue;
            }

            if num > other_num {
                return Ordering::Greater;
            }

            if num < other_num {
                return Ordering::Less;
            }
        }

        Ordering::Equal
    }
}

impl PartialEq for Version {
    /// Checks equality of this `Version` and another `Version`.
    fn eq(&self, other: &Self) -> bool {
        match self.cmp(other) {
            Ordering::Less => false,
            Ordering::Equal => true,
            Ordering::Greater => false,
        }
    }
}

impl PartialOrd for Version {
    /// Gets an ordering between this `Version` and another `Version`.
    /// An ordering can always be found, None is never returned.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// `Hash` implementation for version to match `PartialEq` implementation.
impl Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut hashable_numbers = self.numbers.clone();
        while hashable_numbers.last() == Some(&VersionNumber::from(0)) {
            hashable_numbers.pop();
        }

        hashable_numbers.hash(state);
    }
}

impl Display for Version {
    /// Formats a `Version` into the following format: <version_number>[.version_number]...
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version_string = self.numbers.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(".");
        write!(f, "{}", version_string)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    /// Parses a string into a `Version`.
    /// Could return a `VersionError` error.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.is_empty() {
            return Err(VersionError::NoneError);
        }

        let mut version_parts = Vec::new();
        for num in string.split('.') {
            version_parts.push(VersionNumber::from_str(num)?);
        }

        Ok(Version { numbers: version_parts })
    }
}

/// Implements the from trait for `&[u32]`.
impl From<&[u32]> for Version {
    fn from(value: &[u32]) -> Self {
        Self {
            numbers: value.iter().map(|v| VersionNumber::from(v.clone())).collect(),
        }
    }
}

/// Implements the from trait for `&[u32; N]`.
impl<const N: usize> From<&[u32; N]> for Version {
    fn from(value: &[u32; N]) -> Self {
        Self {
            numbers: value.iter().map(|v| VersionNumber::from(v.clone())).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let correct_version = Version::from(&[3, 4, 1]);

        match Version::from_str("3.4.1") {
            Ok(version) => assert_eq!(version, correct_version),
            Err(e) => panic!("Expected Ok(Version (numbers: [3, 4, 1])), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_none_errors() {
        assert!(matches!(Version::from_str("3.4..1"), Err(VersionError::NoneError)));
        assert!(matches!(Version::from_str("3.4.1."), Err(VersionError::NoneError)));
        assert!(matches!(Version::from_str(".3.4.1"), Err(VersionError::NoneError)));
    }

    #[test]
    fn from_str_no_input() {
        assert!(matches!(Version::from_str(""), Err(VersionError::NoneError)));
    }

    #[test]
    fn from_str_illegal_char() {
        assert!(matches!(Version::from_str("3.a.1"), Err(VersionError::IllegalCharacterError)));
        assert!(matches!(Version::from_str("3.-1.1"), Err(VersionError::IllegalCharacterError)));
    }

    #[test]
    fn compare() {
        let version_a = Version::from(&[3, 4, 0]);
        let version_b = Version::from(&[3, 4, 0]);
        let version_c = Version::from(&[3, 4, 1]);
        let version_d = Version::from(&[3, 3, 5]);

        assert!(version_a == version_b);
        assert!(version_a <= version_b);
        assert!(version_a >= version_b);
        assert!(version_a <= version_c);
        assert!(version_a >= version_d);
        assert!(version_a < version_c);
        assert!(version_a > version_d);
        assert!(version_a != version_c);
    }

    #[test]
    fn compare_different_length() {
        let version_a = Version::from(&[3, 4, 0, 0]);
        let version_b = Version::from(&[3, 4, 0]);
        let version_c = Version::from(&[4]);
        let version_d = Version::from(&[3]);
        let version_e = Version::from(&[0, 3, 3, 5]);
        let version_f = Version::from(&[3, 3, 5]);

        assert!(version_a == version_b);
        assert!(version_c > version_b);
        assert!(version_d > version_e);
        assert!(version_f > version_e);
    }

    #[test]
    fn format() {
        let version = Version::from(&[3, 4, 1]);

        assert_eq!(version.to_string(), "3.4.1");
    }
}
