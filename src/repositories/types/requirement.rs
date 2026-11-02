// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    installer::types::{VersionError, VersionIntervals},
    platforms::tool_detection,
};

/// Errors that occur when parsing requirements.
#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("Requirement '{0}' is unknown")]
    InvalidRequirement(String),

    #[error("Expected a version, because '@' was used")]
    ExpectedVersion,

    #[error("Invalid version")]
    VersionError(#[from] VersionError),
}

/// Represents requirements of a build or test process of a package.
#[derive(Clone, Debug)]
pub enum Requirement {
    Msvc {
        version_intervals: VersionIntervals,
    },
}

impl FromStr for Requirement {
    type Err = RequirementError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Check MSVC
        if string == "msvc" || string.starts_with("msvc@") {
            // Parse out the version
            let version = match string.split_once('@') {
                Some((_, "")) => return Err(RequirementError::ExpectedVersion),
                Some((_, version)) => version,
                None => "",
            };

            let version_intervals = VersionIntervals::from_str(version)?;
            return Ok(Self::Msvc { version_intervals });
        }

        Err(RequirementError::InvalidRequirement(string.into()))
    }
}

impl Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Requirement::Msvc { version_intervals } => write!(f, "msvc@{version_intervals}"),
        }
    }
}

impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = serde::de::Deserialize::deserialize(deserializer)?;

        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Requirement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Requirement {
    /// Checks if a requirement is satisfied.
    /// Returns true if the requirement is satisfied, false otherwise.
    pub fn is_satisfied(&self) -> tool_detection::error::Result<bool> {
        match self {
            Requirement::Msvc { version_intervals } => {
                let Some(msvc) = tool_detection::detect_msvc()? else {
                    return Ok(false);
                };

                Ok(version_intervals.covers(msvc.get_version()))
            },
        }
    }

    /// Gets the message that should be shown when the requirement is not satisfied.
    pub fn get_not_satisfied_message(&self) -> &str {
        match self {
            Requirement::Msvc { .. } => {
                "The required Microsoft Visual C++ cannot be found, please install it from the Microsoft website first"
            },
        }
    }
}
