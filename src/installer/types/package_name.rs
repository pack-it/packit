use std::{fmt::Display, ops::Deref, path::Path, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize, de};

use crate::installer::types::PackageIdError;

const VALID_PACKAGE_NAME: &str = r"^[a-zA-Z0-9\-_]+$";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageName(String);

impl<'de> Deserialize<'de> for PackageName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(&string).map_err(de::Error::custom)?)
    }
}

impl Serialize for PackageName {
    /// Serializes the PackageName. Note that this doesn't check its validity, it assumes
    /// that the PackageName validity is always checked upon creation.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)?;
        Ok(())
    }
}

impl FromStr for PackageName {
    type Err = PackageIdError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(VALID_PACKAGE_NAME).expect("Expected valid regex");
        if !re.is_match(string) {
            return Err(PackageIdError::InvalidPackageName);
        }

        return Ok(Self(string.to_string()));
    }
}

impl Deref for PackageName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for PackageName {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}
