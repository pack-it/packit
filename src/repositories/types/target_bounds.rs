use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    installer::types::{VersionError, VersionIntervals},
    platforms::{Os, OsVersion, Target, TargetArchitecture},
};

/// Errors that occur when creating or using the target bounds.
#[derive(Error, Debug)]
pub enum TargetBoundsError {
    #[error("Target additions are not allowed for this target name")]
    AdditionNotAllowed,

    #[error("Version bounds are not allowed for this target name")]
    VersionBoundsNotAllowed,

    #[error("Target name is invalid")]
    InvalidTargetName,

    #[error("Cannot parse version number")]
    VersionError(#[from] VersionError),
}

/// Represents a target, a target can be a group (e.g. Unix for MacOs and Linux), an operating system or a specific architecture.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetName {
    Architecture(TargetArchitecture),
    Os(Os),
    Unix,
}

impl FromStr for TargetName {
    type Err = TargetBoundsError;

    /// Converts a string to a TargetName struct. A `TargetBoundsError::InvalidTargetName` is
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

impl TargetName {
    /// Checks if the target name is a Unix target. Returns true if it is, false otherwise.
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

/// Represents the bounds of a target. Specifying its name, optionally an addition (e.g. Linux distro) and possible versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetBounds {
    pub name: TargetName,
    pub addition: Option<String>,
    pub version_intervals: VersionIntervals,
}

impl<'de> Deserialize<'de> for TargetBounds {
    /// Deserializes a string into a TargetBounds struct.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(&string).map_err(serde::de::Error::custom)?)
    }
}

impl FromStr for TargetBounds {
    type Err = TargetBoundsError;

    /// Parses a string into a TargetBounds struct. Returns an error if the version intervals or target name are invalid.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Split name and version_bounds
        let (name, version_bounds) = match string.split_once('@') {
            Some(val) => val,
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
        if matches!(name, TargetName::Unix) && !version_bounds.is_empty() {
            return Err(TargetBoundsError::VersionBoundsNotAllowed);
        }

        Ok(Self {
            name: name.into(),
            addition: addition.map(|x| x.into()),
            version_intervals,
        })
    }
}

impl TargetBounds {
    /// Checks if the bounds of the current target are satisfied by a specific target. Returns true if they are, false otherwise.
    pub fn satisfied_by(&self, target: &Target) -> bool {
        // Check if target name matches
        match &self.name {
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
    /// have a higher priority and vice versa (e.g. 'MacOs' has a higher priority then 'Unix').
    fn calculate_priority(&self) -> u32 {
        if self.addition.is_none() && self.version_intervals.is_empty() {
            match self.name {
                TargetName::Unix => return 1,
                TargetName::Os(_) => return 2,
                _ => return 3,
            }
        }

        if self.addition.is_none() && !self.version_intervals.is_empty() {
            match self.name {
                TargetName::Os(_) => return 4,
                _ => return 5,
            }
        }

        if self.addition.is_some() && !self.version_intervals.is_empty() {
            match self.name {
                TargetName::Os(_) => return 6,
                _ => return 7,
            }
        }

        0
    }

    /// Gets the best satisfying target bound. The best meaning the bound with the highest priority.
    /// None will be returned if no satisfying target bounds can be found.
    pub fn get_best_target<'a>(specific_target: &Target, targets: Vec<&'a TargetBounds>) -> Option<&'a TargetBounds> {
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

        current_best
    }
}
