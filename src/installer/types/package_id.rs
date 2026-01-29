use std::{fmt::Display, str::FromStr};

use serde::{de, Deserialize, Serialize};

use crate::installer::types::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub version: Version,
}

impl<'de> Deserialize<'de> for PackageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        let index = string.chars().position(|c| c == '@');

        let (name, version) = match index {
            Some(index) => string.split_at(index),
            None => panic!("Error should have version specified"),
        };

        // Remove @ character from version number
        let version = Version::from_str(version.strip_prefix("@").unwrap_or("")).map_err(de::Error::custom)?;

        Ok(Self {
            name: name.to_string(),
            version,
        })
    }
}

impl Serialize for PackageId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", &self.name, &self.version)?;
        Ok(())
    }
}
