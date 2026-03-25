use std::{fmt::Display, str::FromStr};

use crate::installer::types::{PackageId, PackageName, Version, package_id::PackageIdError};

/// An optional package id, which holds a package name and optionally a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalPackageId {
    pub name: PackageName,
    pub version: Option<Version>,
}

impl From<PackageId> for OptionalPackageId {
    /// Creates an `OptionalPackageId` from a `PackageId`.
    fn from(value: PackageId) -> Self {
        Self {
            name: value.name,
            version: Some(value.version),
        }
    }
}

impl FromStr for OptionalPackageId {
    type Err = PackageIdError;

    /// Parses a string into an `OptionalPackageId`.
    /// Could return a `PackageIdError`.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.contains("@") {
            let package_id = PackageId::from_str(&string)?;

            return Ok(Self {
                name: package_id.name,
                version: Some(package_id.version),
            });
        }

        Ok(Self {
            name: PackageName::from_str(string)?,
            version: None,
        })
    }
}

impl Display for OptionalPackageId {
    /// Formats the `OptionalPackageId` into the following format: <name>[@version].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;

        if let Some(version) = &self.version {
            write!(f, "@{}", version)?;
        }

        Ok(())
    }
}

impl OptionalPackageId {
    /// Returns a `PackageId` or None if the version is not specified in `Self`.
    pub fn versioned(&self) -> Option<PackageId> {
        match &self.version {
            Some(version) => Some(PackageId::new(self.name.clone(), version.clone())),
            None => None,
        }
    }

    /// Returns a `PackageId` with the current version, or the given version if the `OptionalPackageId` does not contain a version.
    pub fn versioned_or(&self, version: Version) -> PackageId {
        let version = match &self.version {
            Some(version) => version.clone(),
            None => version,
        };

        PackageId::new(self.name.clone(), version)
    }
}

#[cfg(test)]
mod tests {
    use crate::installer::types::{PackageId, Version, package_id::PackageIdError, package_name::PackageNameError};

    use super::*;

    #[test]
    fn from_str_optional() {
        let package_name = PackageName::from_str("test").expect("Expected valid package name.");
        let version = Version::from_str("3.4.1").expect("Expected Version.");
        let correct_version = PackageId::new(package_name.clone(), version).into();
        match OptionalPackageId::from_str("test@3.4.1") {
            Ok(id) => assert_eq!(id, correct_version),
            Err(e) => panic!("Expected Ok(OptionalPackageId(name: 'test', version: Some(Version(..)))), got Err({e:?})"),
        }

        let correct_version = OptionalPackageId {
            name: package_name,
            version: None,
        };
        match OptionalPackageId::from_str("test") {
            Ok(id) => assert_eq!(id, correct_version),
            Err(e) => panic!("Expected Ok(OptionalPackageId(name: 'test', version: None)), got Err({e:?})"),
        }
    }

    #[test]
    fn from_str_empty_optional() {
        assert!(matches!(
            OptionalPackageId::from_str(""),
            Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
        ))
    }

    #[test]
    fn from_str_invalid_chars() {
        let invalid_chars = "!#$%^&*()~:;{}[]<>,.?/|\\\"\'`+=";
        for char in invalid_chars.chars() {
            assert!(matches!(
                OptionalPackageId::from_str(format!("{char}@3.4.1").as_str()),
                Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
            ))
        }
    }
}
