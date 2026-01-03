use std::str::FromStr;

use crate::installer::types::{DependencyParserError, Version};

#[derive(Debug, Clone)]
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
                return Ok(VersionBounds::Range(Version::from_str(lower)?, Version::from_str(upper)?));
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
        let ranges = ranges.split('|');
        let mut bounds = Vec::new();

        for range in ranges {
            bounds.push(VersionBounds::from_str(range)?);
        }

        // Bounds must have at least one item
        if bounds.is_empty() {
            return Err(DependencyParserError::EmptyBoundsError);
        }

        Ok(bounds)
    }
}
