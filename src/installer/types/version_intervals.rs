// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de};

use crate::installer::types::{Version, VersionBounds, VersionError};

/// Represents multiple `VersionBounds`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct VersionIntervals {
    version_bounds: Vec<VersionBounds>,
}

impl<'de> Deserialize<'de> for VersionIntervals {
    /// Deserializes a string into `VersionIntervals`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;

        Self::from_str(&string).map_err(de::Error::custom)
    }
}

impl Serialize for VersionIntervals {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl FromStr for VersionIntervals {
    type Err = VersionError;

    /// Parses a string into `VersionIntervals`.
    /// Could return a `VersionError` error.
    fn from_str(intervals: &str) -> Result<Self, Self::Err> {
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

        // Check for invalid intervals
        if !Self::bounds_valid(&version_bounds) {
            return Err(VersionError::InvalidInterval);
        }

        Ok(Self { version_bounds })
    }
}

impl Display for VersionIntervals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let bounds: Vec<_> = self.version_bounds.iter().map(|x| x.to_string()).collect();
        write!(f, "{}", bounds.join("|"))
    }
}

impl VersionIntervals {
    /// Checks if the intervals are valid, the intervals are valid if they don't overlap and are in order.
    /// True is returned if the intervals are valid, otherwise false.
    fn bounds_valid(version_bounds: &Vec<VersionBounds>) -> bool {
        let mut previous: Option<&VersionBounds> = None;
        for bound in version_bounds {
            let valued_previous = match previous {
                Some(previous) => previous,
                None => {
                    previous = Some(bound);
                    continue;
                },
            };

            // In this match we return false early if the bound is a lower (or lower equal) bound.
            // We can do this, because the first iteration never gets here. Meaning that if we have
            // a lower bound the intervals are either not in order or are overlapping.
            let low_version = match bound {
                VersionBounds::Range(low, _) => low,
                VersionBounds::IncludingRange(low, _) => low,
                VersionBounds::Lower(_) => return false,
                VersionBounds::LowerEqual(_) => return false,
                VersionBounds::Higher(version) => version,
                VersionBounds::HigherEqual(version) => version,
                VersionBounds::Equal(version) => version,
            };

            // Here we don't have to compare each bound type with each other bound type, again because the intervals
            // need to be in order.
            match valued_previous {
                VersionBounds::Range(_, high) if *low_version < *high => return false,
                VersionBounds::IncludingRange(_, high) if *low_version <= *high => return false,
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

    /// Checks if any of the version bounds in the current version interval cover a given version.
    /// Returns true if it does, false otherwise.
    pub fn covers(&self, version: &Version) -> bool {
        // If version bounds are empty, version satisfies the bounds
        if self.is_empty() {
            return true;
        }

        // Check if any of the version bounds covers the version
        for bound in &self.version_bounds {
            if bound.covers(version) {
                return true;
            }
        }

        false
    }

    /// Checks if the version bounds are empty. Returns true if it is, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.version_bounds.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::installer::types::version::tests::create_version;

    use super::*;

    /// This is a helper method which creates a `VersionIntervals` from an `intervals_str`  which is assumed to be correct.
    pub fn create_version_intervals(intervals_str: &str) -> VersionIntervals {
        VersionIntervals::from_str(intervals_str).expect("Expected valid version intervals string")
    }

    #[test]
    fn from_str_ranges() {
        let version_intervals = VersionIntervals::from_str("<6.6|6.7|6.8-7.10|>8");
        assert_eq!(
            version_intervals,
            Ok(VersionIntervals {
                version_bounds: vec![
                    VersionBounds::Lower(create_version("6.6")),
                    VersionBounds::Equal(create_version("6.7")),
                    VersionBounds::Range(create_version("6.8"), create_version("7.10")),
                    VersionBounds::Higher(create_version("8"))
                ]
            })
        );
    }

    #[test]
    fn from_str_ranges_empty() {
        let version_intervals = VersionIntervals::from_str("");
        assert_eq!(version_intervals, Ok(VersionIntervals { version_bounds: vec![] }));
    }

    #[test]
    fn from_str_valid_interval() {
        let intervals = ["3", "<6.6|6.7|6.8-7.10|>8", "<=4|4.5|5-6|>=10.1", "4-10", "32|>34", "<6.5|>6.5"];
        for interval in intervals {
            let parsed_interval = VersionIntervals::from_str(interval);
            assert!(
                parsed_interval.is_ok(),
                "{}\nTest value: {interval:?}",
                parsed_interval.unwrap_err()
            );
        }
    }

    #[test]
    fn from_str_invalid_interval() {
        let intervals = ["3|3", "5-10|7-11", "<6.5|>=6.4", "<6.6|6.9|6.8-7.10|>8", ">4|5", "4|3"];
        for interval in intervals {
            let parsed_interval = VersionIntervals::from_str(interval);
            assert_eq!(parsed_interval, Err(VersionError::InvalidInterval));
        }
    }

    #[test]
    fn from_str_empty_version() {
        let parsed_interval = VersionIntervals::from_str("||");
        assert_eq!(parsed_interval, Err(VersionError::NoneError));
    }

    #[test]
    fn empty_format() {
        let version_intervals = VersionIntervals {
            version_bounds: Vec::new(),
        };

        assert_eq!(version_intervals.to_string(), "");
    }

    #[test]
    fn format() {
        let version_intervals_str = "<6.6|6.7|6.8-7.10|>8";
        let version_intervals = create_version_intervals(version_intervals_str);
        assert_eq!(version_intervals.to_string(), version_intervals_str);
    }

    #[test]
    fn empty_covers() {
        let version_intervals = VersionIntervals {
            version_bounds: Vec::new(),
        };

        version_intervals.covers(&create_version("0.0.0"));
    }

    #[test]
    fn one_covers() {
        let version_intervals_str = "<6.6|6.7|6.8-7.10|>8";
        let version_intervals = create_version_intervals(version_intervals_str);

        assert!(version_intervals.covers(&create_version("6.9")));
    }

    #[test]
    fn multiple_cover() {
        let version_intervals_str = "<=4|4.5|5-6|>=10.1";
        let version_intervals = create_version_intervals(version_intervals_str);

        assert!(version_intervals.covers(&create_version("0.0.1")));
        assert!(version_intervals.covers(&create_version("4.5")));
        assert!(version_intervals.covers(&create_version("5.5")));
        assert!(version_intervals.covers(&create_version("10.1")));
    }

    #[test]
    fn none_covers() {
        let version_intervals_str = "<6.6|6.7|6.8-7.10|>8";
        let version_intervals = create_version_intervals(version_intervals_str);

        assert!(!version_intervals.covers(&create_version("6.6")));
        assert!(!version_intervals.covers(&create_version("7.10")));
        assert!(!version_intervals.covers(&create_version("8")));
    }
}
