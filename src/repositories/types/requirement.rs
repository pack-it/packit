// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::platforms::tool_detection;

/// Errors that occur when parsing requirements.
#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("Requirement '{0}' is unknown")]
    InvalidRequirement(String),
}

/// Represents requirements of a build or test process of a package.
#[derive(Clone, Debug)]
pub enum Requirement {
    Msvc,
}

impl FromStr for Requirement {
    type Err = RequirementError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "msvc" => Ok(Self::Msvc),
            _ => Err(RequirementError::InvalidRequirement(string.into())),
        }
    }
}

impl Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Requirement::Msvc => write!(f, "msvc"),
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
            Requirement::Msvc => Ok(tool_detection::detect_msvc()?.is_some()),
        }
    }

    /// Gets the message that should be shown when the requirement is not satisfied.
    pub fn get_not_satisfied_message(&self) -> &str {
        match self {
            Requirement::Msvc => "Microsoft Visual C++ cannot be found, please install it from the Microsoft website first.",
        }
    }
}
