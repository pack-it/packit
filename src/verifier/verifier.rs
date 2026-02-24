use std::{fs, str::FromStr};

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::types::{PackageId, Version},
    packager,
    repositories::types::Checksum,
    storage::package_register::PackageRegister,
    verifier::{
        error::{Result, VerifierError},
        Issue,
    },
};

/// An enum which represents a specific check.
enum Check {
    StorageConsistency,
    RegisterConsistency,
    DependencyTree,
    Alterations,
}

/// Defines the correct order to do verifier checks.
const VERIFY_ORDER: &[Check] = &[
    Check::StorageConsistency,
    Check::RegisterConsistency,
    Check::Alterations,
    Check::DependencyTree,
];

/// Verifier which scans the Packit environment for issues.
pub struct Verifier<'a> {
    config: &'a Config,
    current_issue: usize,
    issues_found: bool,
}

impl<'a> Verifier<'a> {
    /// Creates a new verifier.
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            current_issue: 0,
            issues_found: false,
        }
    }

    /// Gets the next issue in the correct order, defined in VERIFY_ORDER.
    /// None is returned if there are no more issues to return.
    pub fn next_issue(&mut self, register: &PackageRegister) -> Result<Option<Issue>> {
        loop {
            let check = match VERIFY_ORDER.get(self.current_issue) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase current issue
            self.current_issue += 1;

            let issue = match check {
                Check::StorageConsistency => self.check_storage_consistency(register)?,
                Check::RegisterConsistency => self.check_register_consistency(register)?,
                Check::DependencyTree => self.check_dependency_tree(register),
                Check::Alterations => {
                    warning!("This is an experimental check, issues from this check could be inaccurate.");
                    self.check_alterations(register)?
                },
            };

            if let Some(issue) = issue {
                self.issues_found = true;
                return Ok(Some(issue));
            }
        }
    }

    /// Gets the next issue for a specific package in the correct order, defined in VERIFY_ORDER.
    /// None is returned if there are no more issues to return.
    /// Note that this method assumes that the package exists.
    pub fn next_package_issue(&mut self, package_id: &PackageId, register: &PackageRegister) -> Result<Option<Issue>> {
        loop {
            let check = match VERIFY_ORDER.get(self.current_issue) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase current issue
            self.current_issue += 1;

            let issue = match check {
                Check::StorageConsistency if self.package_storage_is_consistent(package_id)? => continue,
                Check::StorageConsistency => Issue::InconsistentStorage(vec![package_id.clone()]),
                Check::RegisterConsistency if self.register_package_is_consistent(package_id, register)? => continue,
                Check::RegisterConsistency => Issue::InconsistentRegister(vec![package_id.clone()]),
                Check::DependencyTree => {
                    let missing_dependencies = self.check_package_dependency_tree(package_id, register);
                    if missing_dependencies.is_empty() {
                        continue;
                    }

                    Issue::BrokenTree(missing_dependencies)
                },
                Check::Alterations if !self.check_package_alterations(package_id)? => continue,
                Check::Alterations => Issue::AlteredPackage(vec![package_id.clone()]),
            };

            self.issues_found = true;
            return Ok(Some(issue));
        }
    }

    /// Checks for alterations in all packages using a checksum which is compared to the checksum from the pre-build.
    /// Returns an alteration issue or None if there are no packages which are altered.
    fn check_alterations(&self, register: &PackageRegister) -> Result<Option<Issue>> {
        // Find issues with a package, maybe even package it and compare it with checksum
        let mut altered = Vec::new();
        for package in register.iterate_all() {
            if self.check_package_alterations(&package.package_id)? {
                altered.push(package.package_id.clone());
            }
        }

        if altered.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::AlteredPackage(altered)))
    }

    /// Checks for alterations in a single package using a checksum which is compared to the checksum from the pre-build.
    /// Returns true if the package was altered, false if not.
    fn check_package_alterations(&self, package_id: &PackageId) -> Result<bool> {
        let install_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
        let compressed = packager::compress(&install_directory)?;
        let checksum = Checksum::from_bytes(&compressed);

        // TODO: Actually search for the correct checksum
        let correct_checksum = Checksum {
            sha256: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        };

        Ok(checksum != correct_checksum)
    }

    /// Checks if all packages in the register also exist in the Packit package storage.
    /// Returns a storage consistency issue or None if there are no packages missing from storage.
    fn check_storage_consistency(&self, register: &PackageRegister) -> Result<Option<Issue>> {
        let mut missing = Vec::new();
        for package in register.iterate_all() {
            if !self.package_storage_is_consistent(&package.package_id)? {
                missing.push(package.package_id.clone());
            }
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InconsistentStorage(missing)))
    }

    /// Checks if a specific package exists in storage. Note that it doesn't check if the package also exists in the register.
    /// Returns false if the package storage isn't consistent, true if it is.
    fn package_storage_is_consistent(&self, package_id: &PackageId) -> Result<bool> {
        let installed_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Check if the directory exists, if so return true
        if fs::exists(installed_directory)? {
            return Ok(true);
        }

        Ok(false)
    }

    /// Checks if all packages in storage also exist in the Packit register.
    /// Returns a register consistency issue or None if there are packages missing from the register.
    fn check_register_consistency(&self, register: &PackageRegister) -> Result<Option<Issue>> {
        let package_directory = self.config.prefix_directory.join("packages");
        let mut missing = Vec::new();
        for file_package in fs::read_dir(package_directory)? {
            let file_package = file_package?;
            if !file_package.path().is_dir() {
                continue;
            }

            // Get the package name
            let package_name = file_package.file_name();
            let package_name = package_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?;

            for file_version in fs::read_dir(file_package.path())? {
                let file_version = file_version?;
                if !file_version.path().is_dir() {
                    continue;
                }

                // Get the version, and create the package id
                let version = Version::from_str(file_version.file_name().to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;
                let package_id = PackageId::new(package_name, &version)?;

                // Check if the package version also exists in the register, if not add it to missing
                if register.get_package_version(&package_id).is_none() {
                    missing.push(package_id);
                }
            }
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InconsistentRegister(missing)))
    }

    /// Checks if a specific package exists in the register. Note that it doesn't check if the package also exists in storage.
    /// Returns false if the package register isn't consistent, true if it is.
    fn register_package_is_consistent(&self, package_id: &PackageId, register: &PackageRegister) -> Result<bool> {
        // Return no issue if the package exists in the register
        if register.get_package_version(package_id).is_some() {
            return Ok(true);
        }

        // Return an inconsistent register issue if the package exists in storage, but not in the register
        Ok(false)
    }

    /// Checks the completeness of the depedency trees from the packages.
    /// Returns a dependency tree issue or None if there are no packages missing from the dependency trees.
    fn check_dependency_tree(&self, register: &PackageRegister) -> Option<Issue> {
        let mut all_missing = Vec::new();
        for package in register.iterate_all() {
            all_missing.extend(self.check_package_dependency_tree(&package.package_id, register));
        }

        if all_missing.is_empty() {
            return None;
        }

        Some(Issue::BrokenTree(all_missing))
    }

    /// Checks the completeness of the dependency tree from a specific package.
    /// Returns a list of missing packages, can be empty if there are no packages missing from the tree.
    fn check_package_dependency_tree(&self, package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
        let package = match register.get_package_version(package_id) {
            Some(package) => package,
            None => {
                warning!("Parent node '{package_id}' doesn't exist, while checking dependency tree.");
                return Vec::new();
            },
        };

        let mut missing = Vec::new();
        for dependency in &package.dependencies {
            if register.get_package_version(&dependency).is_none() {
                missing.push((package_id.clone(), dependency.clone()));
                continue;
            }

            // Recursively check if all the dependencies are installed
            missing.extend(self.check_package_dependency_tree(dependency, register));
        }

        missing
    }

    /// Get the issues found states.
    pub fn issues_found(&self) -> bool {
        self.issues_found
    }
}
