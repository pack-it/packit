use std::str::FromStr;

use serde::{de, Deserialize};
use sha2::{Digest, Sha256};

#[derive(Debug, PartialEq, Eq)]
pub struct Checksum {
    pub sha256: [u8; 32],
}

impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;

        Checksum::from_str(&string).map_err(de::Error::custom)
    }
}

impl FromStr for Checksum {
    type Err = hex::FromHexError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut sha256 = [0; 32];
        hex::decode_to_slice(string, &mut sha256)?;

        Ok(Self { sha256 })
    }
}

impl Checksum {
    pub fn from_bytes(buffer: &mut Vec<u8>) -> Self {
        let checksum: [u8; 32] = Sha256::digest(buffer).into();
        Self { sha256: checksum }
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.sha256)
    }
}
