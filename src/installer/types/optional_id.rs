// SPDX-License-Identifier: GPL-3.0-only
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
            let package_id = PackageId::from_str(string)?;

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
        write!(f, "{}", self.name)?;

        if let Some(version) = &self.version {
            write!(f, "@{}", version)?;
        }

        Ok(())
    }
}

impl OptionalPackageId {
    /// Returns a `PackageId` or `None` if the version is not specified in `Self`.
    #[expect(clippy::manual_map)]
    pub fn versioned(&self) -> Option<PackageId> {
        match &self.version {
            Some(version) => Some(PackageId::new(self.name.clone(), version.clone())),
            None => None,
        }
    }

    /// Returns a `PackageId` with the current version, or the given version if the `OptionalPackageId` does not contain a version.
    #[cfg_attr(not(test), expect(unused))]
    pub fn versioned_or(&self, version: Version) -> PackageId {
        let version = match &self.version {
            Some(version) => version.clone(),
            None => version,
        };

        PackageId::new(self.name.clone(), version)
    }

    /// Returns a `PackageId` with the current version, or a clone of the given version if the `OptionalPackageId` does not contain a version.
    pub fn versioned_or_cloned(&self, version: &Version) -> PackageId {
        let version = match &self.version {
            Some(version) => version,
            None => version,
        };

        PackageId::new(self.name.clone(), version.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::installer::types::{
        PackageId, VersionError,
        package_id::{PackageIdError, tests::create_package_id},
        package_name::{PackageNameError, tests::create_package_name},
        version::tests::create_version,
    };

    use super::*;

    /// This is a helper method which creates an `OptionalPackageId` from an id string which is assumed to be correct.
    pub fn create_optional_id(id: &str) -> OptionalPackageId {
        OptionalPackageId::from_str(id).expect("Expected a valid optional id str")
    }

    #[test]
    fn valid_from_str() {
        let package_name = create_package_name("test");
        let version = create_version("3.4.1");
        let correct_version = PackageId::new(package_name.clone(), version).into();
        assert_eq!(OptionalPackageId::from_str("test@3.4.1"), Ok(correct_version));

        let correct_version = OptionalPackageId {
            name: package_name,
            version: None,
        };
        assert_eq!(OptionalPackageId::from_str("test"), Ok(correct_version));
    }

    #[test]
    fn invalid_from_str() {
        assert_eq!(
            OptionalPackageId::from_str("test@@3.4.1"),
            Err(PackageIdError::VersionError(VersionError::IllegalCharacterError))
        );

        assert_eq!(
            OptionalPackageId::from_str("test@"),
            Err(PackageIdError::VersionError(VersionError::NoneError))
        );

        assert_eq!(
            OptionalPackageId::from_str("3.4.1"),
            Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
        );
    }

    #[test]
    fn from_str_empty_optional() {
        assert_eq!(
            OptionalPackageId::from_str(""),
            Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
        )
    }

    #[test]
    fn from() {
        let optional_id = create_optional_id("test@3.4.1");

        assert_eq!(optional_id.name, create_package_name("test"));
        assert_eq!(optional_id.version, Some(create_version("3.4.1")));
    }

    #[test]
    fn format() {
        let optional_id = create_optional_id("test@3.4.1");
        assert_eq!(optional_id.to_string(), "test@3.4.1");

        let optional_id = create_optional_id("test");
        assert_eq!(optional_id.to_string(), "test");
    }

    #[test]
    fn versioned() {
        let package_id = create_package_id("test@3.4.1");
        let optional_id = create_optional_id("test@3.4.1");
        assert_eq!(optional_id.versioned(), Some(package_id));

        let optional_id = create_optional_id("test");
        assert_eq!(optional_id.versioned(), None);
    }

    #[test]
    fn versioned_or() {
        let version = create_version("2.0.1");
        let package_id = create_package_id("test@3.4.1");
        let optional_id = create_optional_id("test@3.4.1");
        assert_eq!(optional_id.versioned_or(version.clone()), package_id);

        let package_id = create_package_id("test@2.0.1");
        let optional_id = create_optional_id("test");
        assert_eq!(optional_id.versioned_or(version), package_id);
    }

    #[test]
    fn versioned_or_cloned() {
        let version = create_version("2.0.1");
        let package_id = create_package_id("test@3.4.1");
        let optional_id = create_optional_id("test@3.4.1");
        assert_eq!(optional_id.versioned_or_cloned(&version), package_id);

        let package_id = create_package_id("test@2.0.1");
        let optional_id = create_optional_id("test");
        assert_eq!(optional_id.versioned_or_cloned(&version), package_id);
    }
}
