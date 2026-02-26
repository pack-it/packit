use std::str::FromStr;

use crate::installer::types::{DependencyParserError, Version};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VersionBounds {
    Range(Version, Version),
    Lower(Version),
    LowerEqual(Version),
    Higher(Version),
    HigherEqual(Version),
    Equal(Version),
}

impl FromStr for VersionBounds {
    type Err = DependencyParserError;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        // Check if the statement is a two sided range
        if let Some(index) = version.chars().position(|c| c == '-') {
            if let Some((lower, upper)) = version.split_at_checked(index) {
                // Remove '-' from upper before passing it to Version
                return Ok(VersionBounds::Range(Version::from_str(lower)?, Version::from_str(&upper[1..])?));
            }
        }

        // Check lower equal before lower
        if let Some(version) = version.strip_prefix("<=") {
            return Ok(VersionBounds::LowerEqual(Version::from_str(version)?));
        }

        if let Some(version) = version.strip_prefix('<') {
            return Ok(VersionBounds::Lower(Version::from_str(version)?));
        }

        // Check higher equal before higher
        if let Some(version) = version.strip_prefix(">=") {
            return Ok(VersionBounds::HigherEqual(Version::from_str(version)?));
        }

        if let Some(version) = version.strip_prefix('>') {
            return Ok(VersionBounds::Higher(Version::from_str(version)?));
        }

        return Ok(VersionBounds::Equal(Version::from_str(version)?));
    }
}

impl VersionBounds {
    pub fn from_str_ranges(ranges: &str) -> Result<Vec<VersionBounds>, DependencyParserError> {
        // Check for empty input
        if ranges.is_empty() {
            return Ok(Vec::new());
        }

        let ranges = ranges.split('|');
        let mut bounds = Vec::new();

        for range in ranges {
            bounds.push(VersionBounds::from_str(range)?);
        }

        Ok(bounds)
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

    #[test]
    fn from_str_ranges() {
        let version_bounds = VersionBounds::from_str_ranges(">3.4|<6.5");

        match version_bounds {
            Ok(bounds) => {
                assert!(bounds.len() == 2);
                assert!(matches!(bounds.get(0), Some(VersionBounds::Higher(..))), "bound was {:?}", bounds);
                assert!(matches!(bounds.get(1), Some(VersionBounds::Lower(..))), "bound was {:?}", bounds);
            },
            Err(e) => panic!("Expected Ok(Vec(VersionBound (..))), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_ranges_empty() {
        let version_bounds = VersionBounds::from_str_ranges("");

        match version_bounds {
            Ok(bounds) => assert!(bounds.len() == 0),
            Err(e) => panic!("Expected Ok([]), got Err({e:?})"),
        }
    }
}
