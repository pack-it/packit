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
    #[expect(unused)]
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
            Some(version) => version.clone(),
            None => version.clone(),
        };

        PackageId::new(self.name.clone(), version)
    }
}

#[cfg(test)]
mod tests {
    use crate::installer::types::{
        PackageId,
        package_id::PackageIdError,
        package_name::{PackageNameError, tests::create_package_name},
        version::tests::create_version,
    };

    use super::*;

    #[test]
    fn valid_from_str_optional() {
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
    fn from_str_empty_optional() {
        assert_eq!(
            OptionalPackageId::from_str(""),
            Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
        )
    }

    #[test]
    fn from_str_invalid_chars() {
        let invalid_chars = "!#$%^&*()~:;{}[]<>,.?/|\\\"\'`+=";
        for char in invalid_chars.chars() {
            assert_eq!(
                OptionalPackageId::from_str(format!("{char}@3.4.1").as_str()),
                Err(PackageIdError::PackageNameError(PackageNameError::InvalidPackageName))
            )
        }
    }
}
