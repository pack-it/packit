use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{installer::types::Version, storage::installed_package_version::InstalledPackageVersion};

#[derive(Deserialize, Serialize, Debug)]
pub struct InstalledPackage {
    pub versions: HashMap<Version, InstalledPackageVersion>,
}

impl InstalledPackage {
    pub fn create(package_version: InstalledPackageVersion) -> Self {
        Self {
            versions: HashMap::from([(package_version.package_id.version.clone(), package_version)]),
        }
    }

    pub fn get_package_version(&self, version: &Version) -> Option<&InstalledPackageVersion> {
        self.versions.get(version)
    }

    pub fn get_package_version_mut(&mut self, version: &Version) -> Option<&mut InstalledPackageVersion> {
        self.versions.get_mut(version)
    }

    pub fn get_versions(&self) -> Vec<&InstalledPackageVersion> {
        self.versions.values().collect()
    }

    pub fn get_versions_mut(&mut self) -> Vec<&mut InstalledPackageVersion> {
        self.versions.values_mut().collect()
    }

    pub fn add_package_version(&mut self, package_version: InstalledPackageVersion) {
        self.versions.insert(package_version.package_id.version.clone(), package_version);
    }

    pub fn remove_version(&mut self, version: &Version) {
        self.versions.remove(version);
    }
}
