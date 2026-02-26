use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    installer::types::VersionBounds,
    platforms::{self, Target},
};

/// Errors that occur when creating or using the target bounds.
#[derive(Error, Debug, PartialEq)]
pub enum TargetBoundsError {
    #[error("Target additions are not allowed for the target OS")]
    AdditionNotAllowed,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TargetBounds {
    pub name: String, //TODO: we should probably split name into an architecture enum
    pub addition: Option<String>,
    pub version_bounds: Vec<VersionBounds>,
}

impl<'de> Deserialize<'de> for TargetBounds {
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

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let index = string.chars().position(|c| c == '@');

        let (name, version_bounds) = match index {
            Some(index) => string.split_at(index),
            None => (string, ""),
        };

        let version_bounds = VersionBounds::from_str_ranges(version_bounds).unwrap();

        // TODO: split addition from name
        let addition = None;

        // Check if additions are allowed
        if addition.is_some() {
            if platforms::get_os_name(name) != "linux" {
                return Err(TargetBoundsError::AdditionNotAllowed);
            }
        }

        Ok(Self {
            name: name.into(),
            addition,
            version_bounds,
        })
    }
}

impl TargetBounds {
    pub fn satisfies(&self, target: &Target) -> bool {
        //TODO
        false
    }

    fn calculate_priority(&self) -> u32 {
        if self.addition.is_none() && self.version_bounds.is_empty() {
            match self.name.as_str() {
                "unix" => return 1,
                "macos" | "linux" | "windows" => return 2,
                _ => return 3,
            }
        }

        if self.addition.is_none() && !self.version_bounds.is_empty() {
            match self.name.as_str() {
                "macos" | "linux" | "windows" => return 4,
                _ => return 5,
            }
        }

        if self.addition.is_some() && !self.version_bounds.is_empty() {
            match self.name.as_str() {
                "macos" | "linux" | "windows" => return 6,
                _ => return 7,
            }
        }

        0
    }

    pub fn get_best_target<'a>(specific_target: &Target, targets: Vec<&'a TargetBounds>) -> Option<&'a TargetBounds> {
        let mut current_best = None;
        let mut current_best_priority = 0;

        for target in targets {
            if !target.satisfies(specific_target) {
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
