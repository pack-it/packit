use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{installer::types::Version, repositories::types::PackageMeta, storage::installed_package_version::InstalledPackageVersion};

/// The installed package holds package specific info and a mapping from the installed versions to package versions.
#[derive(Deserialize, Serialize, Debug)]
pub struct InstalledPackage {
    #[serde(flatten)]
    pub versions: HashMap<Version, InstalledPackageVersion>,
    pub symlinked: bool,
    pub active_version: Version,

    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

impl InstalledPackage {
    /// Creates a new installed package with its first entry and package specific data.
    pub fn new(package_version: InstalledPackageVersion, symlinked: bool, package: &PackageMeta) -> Self {
        let version = package_version.package_id.version.clone();
        Self {
            versions: HashMap::from([(version.clone(), package_version)]),
            symlinked,
            active_version: version,
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
    pub fn add_package_version(&mut self, package_version: InstalledPackageVersion, active: bool) {
        // Set the active version if active is true
        if active {
            self.active_version = package_version.package_id.version.clone();
        }

        self.versions.insert(package_version.package_id.version.clone(), package_version);
    }
}
