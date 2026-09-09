// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    installer::types::{PackageName, Version, VersionIntervals},
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        types::{DeprecationInfo, TargetBounds},
    },
};

/// Represents the package metadata, containing package information.
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMeta {
    pub name: PackageName,
    pub description: String,
    pub homepage: Option<String>,
    pub versions: Vec<Version>,
    pub required_packit_version: Option<Version>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub conflicts_with: HashSet<PackageName>,
    pub supported_versions: HashMap<TargetBounds, VersionIntervals>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationInfo>,
}

impl PackageMeta {
    /// Gets a sorted list of all supported versions for the given target.
    pub fn get_supported_versions(&self, target: &Target) -> Result<Vec<&Version>> {
        let target = match TargetBounds::get_best_target(target, self.supported_versions.keys().collect()) {
            Some(target) => target,
            None => return Err(RepositoryError::TargetError),
        };

        let supported = self.supported_versions.get(target).ok_or(RepositoryError::TargetError)?;

        // Get all supported versions and sort them
        let mut versions = Vec::new();
        for version in &self.versions {
            // Continue if the version is not supported by the current target
            if !supported.covers(version) {
                continue;
            }

            versions.push(version);
        }
        versions.sort();

        // Return a support error if no supported versions are found
        if versions.is_empty() {
            return Err(RepositoryError::SupportError(self.name.to_string()));
        }

        Ok(versions)
    }
}

#[cfg(test)]
pub mod tests {

    use std::str::FromStr;

    use crate::{
        installer::types::{
            package_name_tests::create_package_name, version_intervals_test::create_version_intervals, version_tests::create_version,
        },
        platforms::{OsVersion, TargetArchitecture},
    };

    use super::*;

    /// A helper method to create a simple package meta test structure
    fn create_package_meta(versions: Vec<Version>, supported_versions: HashMap<TargetBounds, VersionIntervals>) -> PackageMeta {
        PackageMeta {
            name: create_package_name("test"),
            description: "-".to_string(),
            homepage: Some("-".to_string()),
            versions,
            required_packit_version: None,
            conflicts_with: HashSet::new(),
            supported_versions,
            deprecation: None,
        }
    }

    #[test]
    fn supported_versions() {
        let target = Target {
            architecture: TargetArchitecture::MacOsAarch64,
            os: OsVersion::MacOs {
                version: create_version("3.4.1"),
            },
        };

        let versions = vec![create_version("1"), create_version("2"), create_version("3"), create_version("4")];
        let supported_versions = HashMap::from([
            (TargetBounds::from_str("mac").unwrap(), create_version_intervals("2-=4")),
            (TargetBounds::from_str("unix").unwrap(), create_version_intervals("1-3")),
        ]);
        let package_meta = create_package_meta(versions, supported_versions);

        let versions = package_meta.get_supported_versions(&target).unwrap();
        assert_eq!(versions, vec![&create_version("2"), &create_version("3"), &create_version("4")]);
    }

    #[test]
    fn supported_versions_no_target() {
        let target = Target {
            architecture: TargetArchitecture::MacOsAarch64,
            os: OsVersion::MacOs {
                version: create_version("3.4.1"),
            },
        };

        let versions = vec![create_version("1")];
        let supported_versions = HashMap::new();
        let package_meta = create_package_meta(versions, supported_versions);
        assert!(matches!(
            package_meta.get_supported_versions(&target),
            Err(RepositoryError::TargetError)
        ));
    }

    #[test]
    fn supported_versions_no_match() {
        let target = Target {
            architecture: TargetArchitecture::MacOsAarch64,
            os: OsVersion::MacOs {
                version: create_version("3.4.1"),
            },
        };

        let versions = vec![create_version("1")];
        let supported_versions = HashMap::from([(TargetBounds::from_str("mac").unwrap(), create_version_intervals("2-=4"))]);
        let package_meta = create_package_meta(versions, supported_versions);

        assert!(matches!(
            package_meta.get_supported_versions(&target),
            Err(RepositoryError::SupportError(name)) if name == "test"
        ));
    }
}
