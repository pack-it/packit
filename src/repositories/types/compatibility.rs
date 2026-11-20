// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize, de};
use toml::Value;

use crate::{installer::types::VersionIntervals, repositories::types::TargetBounds};

type CompatibilityMap = HashMap<u32, VersionIntervals>;

/// Represents the package compatibility, listing the different versions that are compatible.
#[derive(Serialize, Clone, Debug)]
pub struct PackageCompatibility {
    #[serde(flatten)]
    default: CompatibilityMap,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    targets: HashMap<TargetBounds, CompatibilityMap>,
}

impl<'de> Deserialize<'de> for PackageCompatibility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, Value> = de::Deserialize::deserialize(deserializer)?;

        let mut default = CompatibilityMap::new();
        let mut targets = HashMap::new();

        for (key, value) in map {
            // Parse entries with a number key
            if let Ok(key) = key.parse() {
                let value = VersionIntervals::deserialize(value).map_err(de::Error::custom)?;
                default.insert(key, value);
                continue;
            }

            // Parse entries with a `TargetBounds` key
            let target = TargetBounds::from_str(&key).unwrap();
            let value = deserialize_compatibility_map(value)?;

            validate_compatibility_map(&value)?;

            targets.insert(target, value);
        }

        validate_compatibility_map(&default)?;

        Ok(Self { default, targets })
    }
}

/// Deserializes a compatibility map from a `Value`.
fn deserialize_compatibility_map<E>(value: Value) -> Result<CompatibilityMap, E>
where
    E: serde::de::Error,
{
    let value: HashMap<String, VersionIntervals> = HashMap::deserialize(value).map_err(de::Error::custom)?;

    value
        .into_iter()
        .map(|(key, value)| match key.parse() {
            Ok(key) => Ok((key, value)),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(&key), &"a non-negative integer")),
        })
        .collect()
}

/// Validates a `CompatibilityMap` by checking if the intervals do not overlap.
/// Returns an `Error` if the intervals overlap.
fn validate_compatibility_map<E>(map: &CompatibilityMap) -> Result<(), E>
where
    E: serde::de::Error,
{
    // Check that intervals do not overlap
    for (key_1, interval_1) in map.iter() {
        for (key_2, interval_2) in map.iter() {
            if key_1 == key_2 {
                continue;
            }

            if interval_1.overlaps(interval_2) {
                return Err(de::Error::invalid_value(
                    de::Unexpected::Other("overlapping compatibility intervals"),
                    &"non-overlapping intervals",
                ));
            }
        }
    }

    Ok(())
}
