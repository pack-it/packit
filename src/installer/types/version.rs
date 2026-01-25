use std::{
    cmp::{max, Ordering},
    fmt::Display,
    num::ParseIntError,
    str::FromStr,
};

use serde::{de, Deserialize, Serialize};
use thiserror::Error;

/// Errors that occur when requesting metadata from a repository.
#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Version number is none while version is requested.")]
    NoneError,

    #[error("Version number contains a character which is not a digit or a dot.")]
    IllegalCharacterError,

    #[error("Multiple consecutive dots are not allowed in version number.")]
    ConsecutiveDotsError,

    #[error("Couldn't parse version number")]
    ParseError(#[from] ParseIntError),
}

#[derive(Debug, Eq, Clone)]
pub struct Version {
    numbers: Vec<usize>,
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        Ok(Version::from_str(&string).map_err(de::Error::custom)?)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let iterations = max(self.numbers.len(), other.numbers.len());
        for i in 0..iterations {
            let num = match self.numbers.get(i) {
                Some(num) => *num,
                None => 0,
            };

            let other_num = match other.numbers.get(i) {
                Some(num) => *num,
                None => 0,
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
    fn eq(&self, other: &Self) -> bool {
        match self.cmp(other) {
            Ordering::Less => false,
            Ordering::Equal => true,
            Ordering::Greater => false,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version_string = self.numbers.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(".");
        write!(f, "{}", version_string)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(version_num: &str) -> Result<Self, Self::Err> {
        if version_num.len() == 0 {
            return Err(VersionError::NoneError);
        }

        let mut version_parts = Vec::new();
        for num in version_num.split('.') {
            if num.is_empty() {
                return Err(VersionError::ConsecutiveDotsError);
            }

            if !num.chars().all(|c| c.is_digit(10)) {
                return Err(VersionError::IllegalCharacterError);
            }

            let parsed_num = num.parse::<usize>()?;
            version_parts.push(parsed_num);
        }

        Ok(Version { numbers: version_parts })
    }
}
