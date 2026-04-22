use std::{cmp::Ordering, fmt::Display, hash::Hash, str::FromStr};

use crate::installer::types::VersionError;

/// Represents a single number in a `Version`.
#[derive(Debug, Eq, Clone)]
pub struct VersionNumber {
    original: String,
    number: u32,
}

impl Ord for VersionNumber {
    /// Compares this version number to another version number and returns an `Ordering`.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.number > other.number {
            return Ordering::Greater;
        }

        if self.number < other.number {
            return Ordering::Less;
        }

        Ordering::Equal
    }
}

impl PartialEq for VersionNumber {
    /// Checks equality of this `VersionNumber` and another `VersionNumber`.
    fn eq(&self, other: &Self) -> bool {
        match self.cmp(other) {
            Ordering::Less => false,
            Ordering::Equal => true,
            Ordering::Greater => false,
        }
    }
}

impl PartialOrd for VersionNumber {
    /// Gets an ordering between this `VersionNumber` and another `VersionNumber`.
    /// An ordering can always be found, None is never returned.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// `Hash` implementation for version to match `PartialEq` implementation.
impl Hash for VersionNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.number.hash(state)
    }
}

impl Display for VersionNumber {
    /// Formats a `VersionNumber` using the original number.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}

impl FromStr for VersionNumber {
    type Err = VersionError;

    /// Parses a string into a `VersionNumber`.
    /// Could return a `VersionError` error.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.is_empty() {
            return Err(VersionError::NoneError);
        }

        if !string.chars().all(|c| c.is_digit(10)) {
            return Err(VersionError::IllegalCharacterError);
        }

        Ok(VersionNumber {
            original: string.to_string(),
            number: string.parse()?,
        })
    }
}

/// Implements the from trait for `u32`.
impl From<u32> for VersionNumber {
    fn from(value: u32) -> Self {
        Self {
            original: value.to_string(),
            number: value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let correct_version = VersionNumber {
            original: String::from("3"),
            number: 3,
        };

        match VersionNumber::from_str("3") {
            Ok(version) => assert_eq!(version, correct_version),
            Err(e) => panic!("Expected Ok(VersionNumber(..)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_leading_zeros() {
        let correct_version = VersionNumber {
            original: String::from("03"),
            number: 3,
        };

        match VersionNumber::from_str("03") {
            Ok(version) => assert_eq!(version, correct_version),
            Err(e) => panic!("Expected Ok(VersionNumber(original: 03, number: 3)), got Err({e:?})"),
        }
    }

    #[test]
    fn compare() {
        let version_a = VersionNumber::from_str("03").unwrap();
        let version_b = VersionNumber::from_str("03").unwrap();
        let version_c = VersionNumber::from_str("0000003").unwrap();
        let version_d = VersionNumber::from_str("02").unwrap();
        let version_e = VersionNumber::from_str("04").unwrap();

        assert!(version_a == version_b);
        assert!(version_a == version_c);
        assert!(version_a <= version_c);
        assert!(version_a > version_d);
        assert!(version_a < version_e);
        assert!(version_d < version_c);
        assert!(version_e > version_c);
    }

    #[test]
    fn format() {
        let version = VersionNumber {
            original: String::from("03"),
            number: 3,
        };

        assert_eq!(version.to_string(), "03");
    }
}
