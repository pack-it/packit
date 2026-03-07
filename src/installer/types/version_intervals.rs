use std::str::FromStr;

use crate::installer::types::{Version, VersionBounds, VersionError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionIntervals {
    version_bounds: Vec<VersionBounds>,
}

impl VersionIntervals {
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

        // Check for impossible requirements, like: >3.1|<3.1
        if !Self::validate_intervals(&version_bounds) {
            return Err(VersionError::InvalidInterval);
        }

        Ok(Self { version_bounds })
    }

    /// Checks if the intervals are valid, the intervals are valid if they don't overlap and are in order.
    /// True is returned if the intervals are valid, otherwise false.
    fn validate_intervals(version_bounds: &Vec<VersionBounds>) -> bool {
        let mut previous: Option<&VersionBounds> = None;
        for bound in version_bounds {
            let valued_previous = match previous {
                Some(previous) => previous,
                None => {
                    previous = Some(&bound);
                    continue;
                },
            };

            let low_version = match bound {
                VersionBounds::Range(low, _) => low,
                VersionBounds::Lower(_) => return false,
                VersionBounds::LowerEqual(_) => return false,
                VersionBounds::Higher(version) => version,
                VersionBounds::HigherEqual(version) => version,
                VersionBounds::Equal(version) => version,
            };

            match valued_previous {
                VersionBounds::Range(_, high) if *low_version <= *high => return false,
                VersionBounds::Lower(version) if low_version < version => return false,
                VersionBounds::LowerEqual(version) if low_version <= version => return false,
                VersionBounds::Higher(_) => return false,
                VersionBounds::HigherEqual(_) => return false,
                VersionBounds::Equal(version) if *low_version <= *version => return false,
                _ => {},
            }

            previous = Some(bound)
        }

        true
    }

    pub fn covers(&self, version: &Version) -> bool {
        // If version bounds are empty, version satisfies the bounds
        if self.version_bounds.is_empty() {
            return true;
        }

        // Check if any of the version bounds covers the version
        for bound in &self.version_bounds {
            if bound.covers(&version) {
                return true;
            }
        }

        false
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
