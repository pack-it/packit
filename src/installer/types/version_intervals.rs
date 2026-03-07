use std::str::FromStr;

use crate::installer::types::{VersionBounds, VersionError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionIntervals {
    version_bounds: Vec<VersionBounds>,
}

impl VersionIntervals {
    // TODO: Check for impossible requirements, like: >3.1|<3.1
    pub fn from_str_intervals(intervals: &str) -> Result<Self, VersionError> {
        // Check for empty input
        if intervals.is_empty() {
            return Ok(Self {
                version_bounds: Vec::new(),
            });
        }

        let intervals = intervals.split('|');
        let mut version_bounds = Vec::new();

        for interval in intervals {
            version_bounds.push(VersionBounds::from_str(interval)?);
        }

        Ok(Self { version_bounds })
    }

    pub fn get_version_bounds(&self) -> &Vec<VersionBounds> {
        &self.version_bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_ranges() {
        let version_intervals = VersionIntervals::from_str_intervals(">3.4|<6.5");

        match version_intervals {
            Ok(intervals) => {
                let version_bounds = intervals.get_version_bounds();
                assert!(version_bounds.len() == 2);
                assert!(
                    matches!(version_bounds.get(0), Some(VersionBounds::Higher(..))),
                    "bound was {version_bounds:?}",
                );
                assert!(
                    matches!(version_bounds.get(1), Some(VersionBounds::Lower(..))),
                    "bound was {version_bounds:?}",
                );
            },
            Err(e) => panic!("Expected Ok(Vec(VersionBound (..))), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_ranges_empty() {
        let version_intervals = VersionIntervals::from_str_intervals("");

        match version_intervals {
            Ok(intervals) => assert!(intervals.get_version_bounds().len() == 0),
            Err(e) => panic!("Expected Ok([]), got Err({e:?})"),
        }
    }
}
