use std::io::{self, Cursor, Read};

use serde::{de, Deserialize};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Checksum {
    pub sha256: [u8; 32],
}

impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        let mut sha256 = [0; 32];
        hex::decode_to_slice(string, &mut sha256).map_err(de::Error::custom)?;

        Ok(Self { sha256 })
    }
}

impl Checksum {
    pub fn calculate_checksum(cursor: &mut Cursor<Vec<u8>>) -> Result<String, io::Error> {
        let mut bytes = Vec::new();
        cursor.read_to_end(&mut bytes)?;
        let checksum: [u8; 32] = Sha256::digest(bytes).into();
        Ok(hex::encode(checksum))
    }
}
