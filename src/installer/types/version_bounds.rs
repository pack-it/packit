// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

use crate::installer::types::{Version, VersionError};

/// Holds different types of version bounds.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub enum VersionBounds {
    Range(Version, Version),
    IncludingRange(Version, Version),
    Lower(Version),
    LowerEqual(Version),
    Higher(Version),
    HigherEqual(Version),
    Equal(Version),
}

impl FromStr for VersionBounds {
    type Err = VersionError;

    /// Parses from a string to `VersionBounds`.
    /// Could return a `VersionError` error.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Check if the statement is a two sided range
        if let Some(index) = string.chars().position(|c| c == '-') {
            if let Some((lower, upper)) = string.split_at_checked(index) {
                if upper.starts_with("-=") {
                    return Ok(VersionBounds::IncludingRange(
                        Version::from_str(lower)?,
                        Version::from_str(&upper[2..])?,
                    ));
                }

                // Remove '-' from upper before passing it to Version
                return Ok(VersionBounds::Range(Version::from_str(lower)?, Version::from_str(&upper[1..])?));
            }
        }

        // Check lower equal before lower
        if let Some(version) = string.strip_prefix("<=") {
            return Ok(VersionBounds::LowerEqual(Version::from_str(version)?));
        }

        if let Some(version) = string.strip_prefix('<') {
            return Ok(VersionBounds::Lower(Version::from_str(version)?));
        }

        // Check higher equal before higher
        if let Some(version) = string.strip_prefix(">=") {
            return Ok(VersionBounds::HigherEqual(Version::from_str(version)?));
        }

        if let Some(version) = string.strip_prefix('>') {
            return Ok(VersionBounds::Higher(Version::from_str(version)?));
        }

        Ok(VersionBounds::Equal(Version::from_str(string)?))
    }
}

impl Display for VersionBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionBounds::Range(lower, upper) => write!(f, "{lower}-{upper}"),
            VersionBounds::IncludingRange(lower, upper) => write!(f, "{lower}-={upper}"),
            VersionBounds::Lower(version) => write!(f, "<{version}"),
            VersionBounds::LowerEqual(version) => write!(f, "<={version}"),
            VersionBounds::Higher(version) => write!(f, ">{version}"),
            VersionBounds::HigherEqual(version) => write!(f, ">={version}"),
            VersionBounds::Equal(version) => write!(f, "{version}"),
        }
    }
}

impl VersionBounds {
    /// Checks if the current version bound covers a given version. Returns true if it does, false otherwise.
    pub fn covers(&self, version: &Version) -> bool {
        match self {
            VersionBounds::Range(low, high) if low <= version && high > version => true,
            VersionBounds::IncludingRange(low, high) if low <= version && high >= version => true,
            VersionBounds::Lower(low) if version < low => true,
            VersionBounds::LowerEqual(low) if version <= low => true,
            VersionBounds::Higher(high) if version > high => true,
            VersionBounds::HigherEqual(high) if version >= high => true,
            VersionBounds::Equal(equal) if version == equal => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_range() {
        let version_bound = VersionBounds::from_str("3.4-4.1");

        match version_bound {
            Ok(bound) => assert!(matches!(bound, VersionBounds::Range(..)), "bound was {:?}", bound),
            Err(e) => panic!("Expected Ok(VersionBound (..)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_lower() {
        let version_bound = VersionBounds::from_str("<3.4");

        match version_bound {
            Ok(bound) => assert!(matches!(bound, VersionBounds::Lower(..)), "bound was {:?}", bound),
            Err(e) => panic!("Expected Ok(VersionBound (..)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_lower_equal() {
        let version_bound = VersionBounds::from_str("<=3.4");

        match version_bound {
            Ok(bound) => assert!(matches!(bound, VersionBounds::LowerEqual(..)), "bound was {:?}", bound),
            Err(e) => panic!("Expected Ok(VersionBound (..)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_higher() {
        let version_bound = VersionBounds::from_str(">3.4");

        match version_bound {
            Ok(bound) => assert!(matches!(bound, VersionBounds::Higher(..)), "bound was {:?}", bound),
            Err(e) => panic!("Expected Ok(VersionBound (..)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_higher_equal() {
        let version_bound = VersionBounds::from_str(">=3.4");

        match version_bound {
            Ok(bound) => assert!(matches!(bound, VersionBounds::HigherEqual(..)), "bound was {:?}", bound),
            Err(e) => panic!("Expected Ok(VersionBound (..)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_equal() {
        let version_bound = VersionBounds::from_str("3.4");

        match version_bound {
            Ok(bound) => assert!(matches!(bound, VersionBounds::Equal(..)), "bound was {:?}", bound),
            Err(e) => panic!("Expected Ok(VersionBound (..)), got Err({e:?})"),
        }
    }
}
