use std::{fmt::Display, str::FromStr};

use crate::installer::types::{PackageId, Version, package_id::PackageIdError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalPackageId {
    pub name: String,
    pub version: Option<Version>,
}

impl From<PackageId> for OptionalPackageId {
    fn from(value: PackageId) -> Self {
        Self {
            name: value.name,
            version: Some(value.version),
        }
    }
}

impl FromStr for OptionalPackageId {
    type Err = PackageIdError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.contains("@") {
            let package_id = PackageId::from_str(&string)?;

            return Ok(Self {
                name: package_id.name,
                version: Some(package_id.version),
            });
        }

        if !PackageId::is_valid_name(string) {
            return Err(PackageIdError::InvalidPackageName);
        }

        Ok(Self {
            name: string.into(),
            version: None,
        })
    }
}

impl Display for OptionalPackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;

        if let Some(version) = &self.version {
            write!(f, "@{}", version)?;
        }

        Ok(())
    }
}

impl OptionalPackageId {
    pub fn versioned(&self) -> Option<PackageId> {
        match &self.version {
            Some(version) => Some(PackageId::new(&self.name, version.clone()).expect("Expected valid name from optional package id")),
            None => None,
        }
    }

    pub fn versioned_or(&self, version: Version) -> PackageId {
        let version = match &self.version {
            Some(version) => version.clone(),
            None => version,
        };

        PackageId::new(&self.name, version).expect("Expected valid name from optional package id")
    }
}

#[cfg(test)]
mod tests {
    use crate::installer::types::{PackageId, Version, package_id::PackageIdError};

    use super::*;

    #[test]
    fn from_str_optional() {
        let version = Version::from_str("3.4.1").expect("Expected Version.");
        let correct_version = PackageId::new("test", version).expect("Expected valid package id").into();
        match OptionalPackageId::from_str("test@3.4.1") {
            Ok(id) => assert_eq!(id, correct_version),
            Err(e) => panic!("Expected Ok(OptionalPackageId(name: 'test', version: Some(Version(..)))), got Err({e:?})"),
        }

        let correct_version = OptionalPackageId {
            name: "test".into(),
            version: None,
        };
        match OptionalPackageId::from_str("test") {
            Ok(id) => assert_eq!(id, correct_version),
            Err(e) => panic!("Expected Ok(OptionalPackageId(name: 'test', version: None)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_empty_optional() {
        assert_eq!(OptionalPackageId::from_str(""), Err(PackageIdError::InvalidPackageName));
    }

    #[test]
    fn from_str_invalid_chars() {
        let invalid_chars = "!#$%^&*()~:;{}[]<>,.?/|\\\"\'`+=";
        for char in invalid_chars.chars() {
            assert_eq!(
                OptionalPackageId::from_str(format!("{char}@3.4.1").as_str()),
                Err(PackageIdError::InvalidPackageName)
            );
        }
    }
}
