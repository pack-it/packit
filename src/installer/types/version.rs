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
#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Version is empty")]
    NoneError,

    #[error("Version number contains a character which is not a digit or a dot")]
    IllegalCharacterError,

    #[error("Invalid version interval, an interval must be ordered and not overlapping")]
    InvalidInterval,

    #[error("Multiple leading, trailing or consecutive dots are not allowed in version number")]
    DotsError,

    #[error("Cannot parse version number")]
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
    /// An ordering can always be found, `None` is never returned.
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
    /// Formats a `Version` into the following format: `<version_number>[.version_number]...`.
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
            let version_number = match VersionNumber::from_str(num) {
                Ok(version_number) => version_number,
                Err(VersionError::NoneError) => return Err(VersionError::DotsError),
                Err(e) => return Err(e),
            };

            version_parts.push(version_number);
        }

        Ok(Version { numbers: version_parts })
    }
}

/// Implements the from trait for `&[u32]`.
impl TryFrom<&[u32]> for Version {
    type Error = VersionError;

    fn try_from(value: &[u32]) -> Result<Self, VersionError> {
        if value.is_empty() {
            return Err(VersionError::NoneError);
        }

        Ok(Self {
            numbers: value.iter().map(|v| VersionNumber::from(*v)).collect(),
        })
    }
}

/// Implements the from trait for `&[u32; N]`.
impl<const N: usize> TryFrom<&[u32; N]> for Version {
    type Error = VersionError;
    fn try_from(value: &[u32; N]) -> Result<Self, VersionError> {
        if value.is_empty() {
            return Err(VersionError::NoneError);
        }

        Ok(Self {
            numbers: value.iter().map(|v| VersionNumber::from(*v)).collect(),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// This is a helper method which creates a `Version` which is assumed to be correct.
    pub fn create_version(version_str: &str) -> Version {
        Version::from_str(version_str).expect("Expected a valid version str")
    }

    #[test]
    fn valid_from_str() {
        let correct_version = create_version("3.4.1");
        assert_eq!(Version::from_str("3.4.1"), Ok(correct_version));
    }

    #[test]
    fn from_str_dot_errors() {
        assert_eq!(Version::from_str("3.4..1"), Err(VersionError::DotsError));
        assert_eq!(Version::from_str("3.4.1."), Err(VersionError::DotsError));
        assert_eq!(Version::from_str(".3.4.1"), Err(VersionError::DotsError));
    }

    #[test]
    fn from_str_empty() {
        assert_eq!(Version::from_str(""), Err(VersionError::NoneError));
    }

    #[test]
    fn from_str_illegal_char() {
        assert_eq!(Version::from_str("3.a.1"), Err(VersionError::IllegalCharacterError));
        assert_eq!(Version::from_str("3.-1.1"), Err(VersionError::IllegalCharacterError));
    }

    #[test]
    fn compare() {
        let version_a = create_version("3.4.0");
        let version_b = create_version("3.4.0");
        let version_c = create_version("3.4.1");
        let version_d = create_version("3.3.5");

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
        let version_a = create_version("3.4.0.0");
        let version_b = create_version("3.4.0");
        let version_c = create_version("4");
        let version_d = create_version("3");
        let version_e = create_version("0.3.3.5");
        let version_f = create_version("3.3.5");

        assert!(version_a == version_b);
        assert!(version_c > version_b);
        assert!(version_d > version_e);
        assert!(version_f > version_e);
    }

    #[test]
    fn ordering() {
        let version_a = create_version("3.4.0.1");
        let version_b = create_version("3.4.0");
        let version_c = create_version("0.4");
        let version_d = create_version("0.0.100");

        let mut version_list = vec![&version_a, &version_b, &version_c, &version_d];
        version_list.sort();

        assert_eq!(*version_list[0], version_d);
        assert_eq!(*version_list[1], version_c);
        assert_eq!(*version_list[2], version_b);
        assert_eq!(*version_list[3], version_a);
    }

    #[test]
    fn format() {
        let version = create_version("3.4.1");
        assert_eq!(version.to_string(), "3.4.1");
    }
}
