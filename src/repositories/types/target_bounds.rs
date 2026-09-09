// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, str::FromStr, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    installer::types::{VersionError, VersionIntervals},
    platforms::{Os, OsVersion, Target, TargetArchitecture},
};

static VALID_ADDITION_NAME: &str = r"^[a-z0-9\-_]+$";
static ADDITION_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(VALID_ADDITION_NAME).expect("Expected valid regex"));

/// Errors that occur when creating or using the target bounds.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub enum TargetBoundsError {
    #[error("Target additions are not allowed for this target name")]
    AdditionNotAllowed,

    #[error("Version bounds are not allowed for this target name")]
    VersionBoundsNotAllowed,

    #[error("Target name is invalid")]
    InvalidTargetName,

    #[error("Expected a version, because '@' was used")]
    ExpectedVersion,

    #[error("Addition name cannot be empty and can only contain characters: 'a-z', '0-9', '-' and '_'")]
    InvalidAdditionName,

    #[error("Cannot parse version number")]
    VersionError(#[from] VersionError),
}

/// Represents a target, a target can be a group (e.g. `Unix` for `MacOs` and `Linux`), an operating system or a specific architecture.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetName {
    Architecture(TargetArchitecture),
    Os(Os),
    Unix,
}

impl FromStr for TargetName {
    type Err = TargetBoundsError;

    /// Converts a string to a `TargetName` struct. A `TargetBoundsError::InvalidTargetName` is
    /// returned if the given string is not a valid target.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string == "unix" {
            return Ok(Self::Unix);
        }

        match string {
            "mac" => return Ok(Self::Os(Os::MacOs)),
            "linux" => return Ok(Self::Os(Os::Linux)),
            "windows" => return Ok(Self::Os(Os::Windows)),
            _ => (),
        }

        let architecture = TargetArchitecture::from_str(string);
        if !architecture.is_unknown() {
            return Ok(Self::Architecture(architecture));
        }

        Err(TargetBoundsError::InvalidTargetName)
    }
}

impl Display for TargetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetName::Architecture(target_architecture) => write!(f, "{target_architecture}"),
            TargetName::Os(os) => write!(f, "{}", os.as_str()),
            TargetName::Unix => write!(f, "unix"),
        }
    }
}

impl TargetName {
    /// Checks if the target name is a Unix target. Returns true if it is, false otherwise.
    #[expect(unused)]
    pub fn is_unix(&self) -> bool {
        match self {
            TargetName::Architecture(architecture) => architecture.get_os().is_unix(),
            TargetName::Os(os) => os.is_unix(),
            TargetName::Unix => true,
        }
    }

    /// Gets the operating system from the target name. Returns an option containing the OS or None if the target
    /// name is too broad to specify a specific OS (e.g. Unix).
    pub fn get_os(&self) -> Option<Os> {
        match self {
            TargetName::Architecture(architecture) => Some(architecture.get_os()),
            TargetName::Os(os) => Some(os.clone()),
            TargetName::Unix => None,
        }
    }
}

/// Represents the name of an addition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetAddition(String);

impl FromStr for TargetAddition {
    type Err = TargetBoundsError;

    /// Parses a string into an `TargetAddition`.
    /// Could return a `TargetBoundsError::InvalidAdditionName` error.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if !ADDITION_NAME_REGEX.is_match(string) {
            return Err(TargetBoundsError::InvalidAdditionName);
        }

        Ok(Self(string.to_string()))
    }
}

impl Display for TargetAddition {
    /// Formats an `TargetAddition` into the following format: <name>.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

/// Represents the bounds of a target. Specifying its name, optionally an addition (e.g. Linux distro) and possible versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetBounds {
    pub name: TargetName,
    pub addition: Option<TargetAddition>,
    pub version_intervals: VersionIntervals,
}

impl<'de> Deserialize<'de> for TargetBounds {
    /// Deserializes a string into a `TargetBounds` struct.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = serde::de::Deserialize::deserialize(deserializer)?;

        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl Serialize for TargetBounds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl FromStr for TargetBounds {
    type Err = TargetBoundsError;

    /// Parses a string into a `TargetBounds` struct. Returns an error if the version intervals or target name are invalid.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Split name and version_bounds
        let (name, version_bounds) = match string.split_once('@') {
            Some(value) if value.1.is_empty() => return Err(TargetBoundsError::ExpectedVersion),
            Some(value) => value,
            None => (string, ""),
        };

        let version_intervals = VersionIntervals::from_str(version_bounds)?;

        // Split addition from name
        let (name, addition) = match name.split_once(':') {
            Some((name, addition)) => (name, Some(addition)),
            None => (name, None),
        };

        let name = TargetName::from_str(name)?;

        // Check if additions are allowed
        if addition.is_some() && !matches!(name.get_os(), Some(Os::Linux)) {
            return Err(TargetBoundsError::AdditionNotAllowed);
        }

        // Check if version bounds are given for the unix target
        if matches!(name, TargetName::Unix) && !version_intervals.is_empty() {
            return Err(TargetBoundsError::VersionBoundsNotAllowed);
        }

        // Parse the addition
        let addition = match addition {
            Some(addition) => Some(TargetAddition::from_str(addition)?),
            None => None,
        };

        Ok(Self {
            name,
            addition,
            version_intervals,
        })
    }
}

impl Display for TargetBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Add target name
        write!(f, "{}", self.name)?;

        // Add addition if it exists
        if let Some(addition) = &self.addition {
            write!(f, ":{addition}")?;
        }

        // Add version intervals if there are any
        if !self.version_intervals.is_empty() {
            write!(f, "@{}", self.version_intervals)?;
        }

        Ok(())
    }
}

impl TargetBounds {
    /// Checks if the bounds of the current target are satisfied by a specific target. Returns true if they are, false otherwise.
    pub fn satisfied_by(&self, target: &Target) -> bool {
        // Check if target name matches
        match &self.name {
            TargetName::Architecture(TargetArchitecture::Unknown(_)) => return false,
            TargetName::Architecture(architecture) if *architecture != target.architecture => return false,
            TargetName::Os(os) if *os != target.os.get_os() => return false,
            TargetName::Unix if !target.os.get_os().is_unix() => return false,
            _ => (),
        }

        // Extract relevant version from os version info (and check if distro matches if applicable)
        let version = match &target.os {
            OsVersion::MacOs { version } | OsVersion::Windows { version } => version,
            OsVersion::Linux {
                distro,
                distro_version,
                kernel_version,
            } => match &self.addition {
                Some(addition) if addition != distro => return false,
                Some(_) => distro_version,
                None => kernel_version,
            },
            OsVersion::Unknown => return false,
        };

        self.version_intervals.covers(version)
    }

    /// Calculates the priority of the current target bound. The more specific target bound will
    /// have a higher priority and vice versa (e.g. `MacOs` has a higher priority then `Unix`).
    fn calculate_priority(&self) -> u32 {
        if self.addition.is_none() && self.version_intervals.is_empty() {
            match self.name {
                TargetName::Unix => return 1,
                TargetName::Os(_) => return 2,
                TargetName::Architecture(_) => return 3,
            }
        }

        if self.addition.is_none() && !self.version_intervals.is_empty() {
            match self.name {
                TargetName::Os(_) => return 4,
                TargetName::Architecture(_) => return 5,
                TargetName::Unix => {}, // Unix is never reached
            }
        }

        if self.addition.is_some() && self.version_intervals.is_empty() {
            match self.name {
                TargetName::Os(_) => return 6,
                TargetName::Architecture(_) => return 7,
                TargetName::Unix => {}, // Unix is never reached
            }
        }

        if self.addition.is_some() && !self.version_intervals.is_empty() {
            match self.name {
                TargetName::Os(_) => return 8,
                TargetName::Architecture(_) => return 9,
                TargetName::Unix => {}, // Unix is never reached
            }
        }

        0
    }

    /// Gets the best satisfying target bound. The best meaning the bound with the highest priority.
    /// `None` will be returned if no satisfying target bounds can be found.
    pub fn get_best_target<'a>(specific_target: &Target, targets: Vec<&'a TargetBounds>) -> Option<&'a TargetBounds> {
        Self::get_best_target_priority(specific_target, targets).map(|(_, x)| x)
    }

    /// Gets the best satisfying target bound and its priority. A higher priority means a more specific target.
    /// `None` will be returned if no satisfying target bounds can be found.
    pub fn get_best_target_priority<'a>(specific_target: &Target, targets: Vec<&'a TargetBounds>) -> Option<(u32, &'a TargetBounds)> {
        let mut current_best = None;
        let mut current_best_priority = 0;

        for target in targets {
            if !target.satisfied_by(specific_target) {
                continue;
            }

            let priority = target.calculate_priority();
            if priority < current_best_priority {
                continue;
            }

            if priority == current_best_priority {
                warning!("Found two targets that satisfy and have the same priority!");
            }

            current_best = Some(target);
            current_best_priority = priority;
        }

        current_best.map(|x| (current_best_priority, x))
    }
}

#[cfg(test)]
pub mod tests {

    use crate::installer::types::{version_intervals_test::create_version_intervals, version_tests::create_version};

    use super::*;

    #[test]
    fn from_str() {
        assert_eq!(
            TargetBounds::from_str("linux:linuxmint@1.1.1"),
            Ok(TargetBounds {
                name: TargetName::Os(Os::Linux),
                addition: Some(TargetAddition("linuxmint".to_string())),
                version_intervals: create_version_intervals("1.1.1")
            })
        );
    }

    #[test]
    fn from_str_addition_not_allowed() {
        assert_eq!(
            TargetBounds::from_str("mac:linuxmint@1.1.1"),
            Err(TargetBoundsError::AdditionNotAllowed)
        );
        assert_eq!(
            TargetBounds::from_str("windows:linuxmint@1.1.1"),
            Err(TargetBoundsError::AdditionNotAllowed)
        );
        assert_eq!(
            TargetBounds::from_str("unix:linuxmint@1.1.1"),
            Err(TargetBoundsError::AdditionNotAllowed)
        );
    }

    #[test]
    fn from_str_version_for_unix() {
        assert_eq!(
            TargetBounds::from_str("unix@1.1.1"),
            Err(TargetBoundsError::VersionBoundsNotAllowed)
        );
    }

    #[test]
    fn from_str_missing_version() {
        assert_eq!(TargetBounds::from_str("linux@"), Err(TargetBoundsError::ExpectedVersion));
        assert_eq!(TargetBounds::from_str("@"), Err(TargetBoundsError::ExpectedVersion));
    }

    #[test]
    fn from_str_empty_name() {
        assert_eq!(TargetBounds::from_str("@1.1.1"), Err(TargetBoundsError::InvalidTargetName));
        assert_eq!(TargetBounds::from_str(""), Err(TargetBoundsError::InvalidTargetName));
    }

    #[test]
    fn from_str_invalid_addition() {
        assert_eq!(TargetBounds::from_str("linux:@1.1.1"), Err(TargetBoundsError::InvalidAdditionName));
        let illegal_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ ./\\!@#$%^&*():;'\"<>[]{}?|~`±§=+\u{1234}";
        for name in illegal_chars.chars() {
            assert_eq!(
                TargetAddition::from_str(&name.to_string()),
                Err(TargetBoundsError::InvalidAdditionName),
                "expected {name:?} to be invalid"
            );
        }
    }

    #[test]
    fn satisfied_by_name() {
        let target = Target {
            architecture: TargetArchitecture::MacOsAarch64,
            os: OsVersion::MacOs {
                version: create_version("3.4.1"),
            },
        };

        let target_bounds = TargetBounds::from_str("unix").unwrap();
        assert!(target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("mac").unwrap();
        assert!(target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("aarch64-apple-darwin").unwrap();
        assert!(target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("windows").unwrap();
        assert!(!target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("x86_64-pc-windows-msvc").unwrap();
        assert!(!target_bounds.satisfied_by(&target));
    }

    #[test]
    fn satisfied_by_addition() {
        let target = Target {
            architecture: TargetArchitecture::LinuxX86_64Gnu,
            os: OsVersion::Linux {
                distro: TargetAddition("linuxmint".to_string()),
                distro_version: create_version("3.4.1"),
                kernel_version: create_version("1.0.0"),
            },
        };

        let target_bounds = TargetBounds::from_str("linux:linuxmint").unwrap();
        assert!(target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("linux:linuxmint@3.4.1").unwrap();
        assert!(target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("linux@1.0.0").unwrap();
        assert!(target_bounds.satisfied_by(&target));

        let target_bounds = TargetBounds::from_str("linux:linuxmint@3.4.0").unwrap();
        assert!(!target_bounds.satisfied_by(&target));
        let target_bounds = TargetBounds::from_str("linux@3.4.1").unwrap();
        assert!(!target_bounds.satisfied_by(&target));
    }

    #[test]
    fn satisfied_by_unknown_arch() {
        let target = Target {
            architecture: TargetArchitecture::MacOsAarch64,
            os: OsVersion::MacOs {
                version: create_version("3.4.1"),
            },
        };

        let target_bounds = TargetBounds {
            name: TargetName::Architecture(TargetArchitecture::Unknown(None)),
            addition: None,
            version_intervals: create_version_intervals(""),
        };

        assert!(!target_bounds.satisfied_by(&target))
    }

    #[test]
    fn priorities() {
        let unix = TargetBounds::from_str("unix").unwrap().calculate_priority();
        let os = TargetBounds::from_str("mac").unwrap().calculate_priority();
        let arch = TargetBounds::from_str("aarch64-apple-darwin").unwrap().calculate_priority();
        let os_versioned = TargetBounds::from_str("mac@0.0.1").unwrap().calculate_priority();
        let arch_versioned = TargetBounds::from_str("aarch64-apple-darwin@6.9").unwrap().calculate_priority();
        let os_addition = TargetBounds::from_str("linux:linuxmint").unwrap().calculate_priority();
        let arch_addition = TargetBounds::from_str("x86_64-unknown-linux-gnu:linuxmint").unwrap().calculate_priority();
        let os_addition_versioned = TargetBounds::from_str("linux:linuxmint@6.9").unwrap().calculate_priority();
        let arch_addition_versioned = TargetBounds::from_str("x86_64-unknown-linux-gnu:linuxmint@9.9.2026").unwrap().calculate_priority();

        assert!(unix < os);
        assert!(os < arch);
        assert!(arch < os_versioned);
        assert!(os_versioned < arch_versioned);
        assert!(arch_versioned < os_addition);
        assert!(os_addition < arch_addition);
        assert!(arch_addition < os_addition_versioned);
        assert!(os_addition_versioned < arch_addition_versioned);
    }

    #[test]
    fn best_target() {
        let unix = TargetBounds::from_str("unix").unwrap();
        let os = TargetBounds::from_str("mac").unwrap();
        let arch = TargetBounds::from_str("aarch64-apple-darwin").unwrap();
        let os_versioned = TargetBounds::from_str("mac@0.0.1").unwrap();
        let arch_versioned = TargetBounds::from_str("aarch64-apple-darwin@6.9").unwrap();
        let os_addition = TargetBounds::from_str("linux:linuxmint").unwrap();
        let arch_addition = TargetBounds::from_str("x86_64-unknown-linux-gnu:linuxmint").unwrap();
        let os_addition_versioned = TargetBounds::from_str("linux:linuxmint@6.9").unwrap();
        let arch_addition_versioned = TargetBounds::from_str("x86_64-unknown-linux-gnu:linuxmint@9.9.2026").unwrap();

        #[rustfmt::skip]
        let bounds = vec![
            &unix, &os, &arch, &os_versioned, &arch_versioned, &os_addition, &arch_addition, &os_addition_versioned, &arch_addition_versioned
        ];

        let target = Target {
            architecture: TargetArchitecture::MacOsAarch64,
            os: OsVersion::MacOs {
                version: create_version("6.9"),
            },
        };

        assert_eq!(TargetBounds::get_best_target(&target, bounds), Some(&arch_versioned));
    }
}
