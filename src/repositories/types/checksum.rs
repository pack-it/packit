use serde::{de, Deserialize};

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
