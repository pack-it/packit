use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{installer::types::Version, repositories::types::PackageMetadata, storage::installed_package_version::InstalledPackageVersion};

/// The installed package holds package specific info and a mapping from versions to package versions.
#[derive(Deserialize, Serialize, Debug)]
pub struct InstalledPackage {
    pub versions: HashMap<Version, InstalledPackageVersion>,
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

impl InstalledPackage {
    /// Creates a new installed package with its first entry and package specific data.
    pub fn new(package_version: InstalledPackageVersion, package: &PackageMetadata) -> Self {
        Self {
            versions: HashMap::from([(package_version.package_id.version.clone(), package_version)]),
            description: package.description.clone(),
            homepage: package.homepage.clone(),
        }
    }

    /// Gets a package version if it exists.
    /// Returns a reference to the package version or None of the package is not installed.
    pub fn get_package_version(&self, version: &Version) -> Option<&InstalledPackageVersion> {
        self.versions.get(version)
    }

    /// Gets a package version if it exists.
    /// Returns a mutable reference to the package version or None of the package is not installed.
    pub fn get_package_version_mut(&mut self, version: &Version) -> Option<&mut InstalledPackageVersion> {
        self.versions.get_mut(version)
    }

    /// Gets references to all the package versions of the current package (cannot be empty).
    pub fn get_versions(&self) -> Vec<&InstalledPackageVersion> {
        self.versions.values().collect()
    }

    /// Gets mutable references to all the package versions of the current package (cannot be empty).
    pub fn get_versions_mut(&mut self) -> Vec<&mut InstalledPackageVersion> {
        self.versions.values_mut().collect()
    }

    /// Adds a package version.
    pub fn add_package_version(&mut self, package_version: InstalledPackageVersion) {
        self.versions.insert(package_version.package_id.version.clone(), package_version);
    }

    /// Removes a package version.
    pub fn remove_version(&mut self, version: &Version) {
        self.versions.remove(version);
    }
}
