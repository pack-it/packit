// SPDX-License-Identifier: GPL-3.0-only
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de};
use sha2::{Digest, Sha256};

/// Represents a checksum, wraps around a byte array.
#[derive(Debug, PartialEq, Eq)]
pub struct Checksum {
    pub sha256: [u8; 32],
}

impl<'de> Deserialize<'de> for Checksum {
    /// Deserializes a string into a Checksum.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;

        Checksum::from_str(&string).map_err(de::Error::custom)
    }
}

impl Serialize for Checksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl FromStr for Checksum {
    type Err = hex::FromHexError;

    /// Converts a string into a Checksum.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut sha256 = [0; 32];
        hex::decode_to_slice(string, &mut sha256)?;

        Ok(Self { sha256 })
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.sha256))
    }
}

impl Checksum {
    /// Creates a checksum from the given bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let checksum: [u8; 32] = Sha256::digest(bytes).into();
        Self { sha256: checksum }
    }
}
