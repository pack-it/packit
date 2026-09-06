use std::path::Path;

use crate::{
    installer::types::PackageName,
    register::{
        metadata::{LocalMetaHandler, error::Result},
        package_register::PackageRegister,
    },
};

/// Extension trait for local metadata interaction with register.
pub trait PackageRegisterExt {
    /// Checks if a package has conflicts with any package that is installed.
    /// Returns a list of the names of all conflicting packages.
    fn get_conflicting_packages(
        &self,
        package_name: &PackageName,
        package_conflicts: &[PackageName],
        prefix_dir: &Path,
    ) -> Result<Vec<PackageName>>;
}

impl PackageRegisterExt for PackageRegister {
    fn get_conflicting_packages(
        &self,
        package_name: &PackageName,
        package_conflicts: &[PackageName],
        prefix_dir: &Path,
    ) -> Result<Vec<PackageName>> {
        let mut conflicting_packages = Vec::new();

        for (name, package) in self.iterate_packages() {
            if !package.symlinked || name == package_name {
                continue;
            }

            // Check if the package specifies this package as conflict
            if package_conflicts.contains(name) {
                conflicting_packages.push(name.clone());
            }

            // Check if this package specifies the package as conflict
            let conflicts = LocalMetaHandler::read_package_conflicts(package, prefix_dir)?;
            if conflicts.contains(package_name) {
                conflicting_packages.push(name.clone());
            }
        }

        Ok(conflicting_packages)
    }
}
