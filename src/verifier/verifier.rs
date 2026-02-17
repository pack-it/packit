use std::{fs, str::FromStr};

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::types::{PackageId, Version},
    packager,
    repositories::types::Checksum,
    storage::package_register::PackageRegister,
    verifier::{error::VerifierError, Issue},
};

// An enum which represents a specific check
enum Check {
    StorageConsistency,
    RegisterConsistency,
    DependencyTree,
    Alterations,
}

// Define the correct order to do checks
const VERIFY_ORDER: &[Check] = &[
    Check::StorageConsistency,
    Check::RegisterConsistency,
    Check::Alterations,
    Check::DependencyTree,
];

pub struct Verifier<'a> {
    config: &'a Config,
    current_issue: usize,
    issues_found: bool,
}

impl<'a> Verifier<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            current_issue: 0,
            issues_found: false,
        }
    }

    /// Gets the next issue in the correct order, defined in VERIFY_ORDER.
    /// None is returned if there are no more issues to return.
    pub fn next_issue(&mut self, register: &PackageRegister) -> Result<Option<Issue>, VerifierError> {
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
                Check::Alterations => self.check_alterations(register)?,
            };

            if let Some(issue) = issue {
                self.issues_found = true;
                return Ok(Some(issue));
            }
        }
    }

    pub fn next_package_issue(&mut self, package_id: &PackageId, register: &PackageRegister) -> Result<Option<Issue>, VerifierError> {
        loop {
            let check = match VERIFY_ORDER.get(self.current_issue) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase curren issue
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

    pub fn show_all_issues(&mut self, register: &PackageRegister) -> Result<(), VerifierError> {
        while let Some(issue) = self.next_issue(register)? {
            print!("{issue}\n");
        }

        Ok(())
    }

    pub fn show_all_package_issues(&mut self, package_id: &PackageId, register: &PackageRegister) -> Result<(), VerifierError> {
        while let Some(issue) = self.next_package_issue(package_id, register)? {
            print!("{issue}\n");
        }

        Ok(())
    }

    fn check_alterations(&self, register: &PackageRegister) -> Result<Option<Issue>, VerifierError> {
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

    fn check_package_alterations(&self, package_id: &PackageId) -> Result<bool, VerifierError> {
        let install_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
        let mut compressed = packager::compress(&install_directory)?;
        let checksum = Checksum::calculate_checksum(&mut compressed)?;

        // TODO: Actually search for the correct checksum
        let correct_checksum = "904723e939e42c4cfa16b2cc8bf9f7d637a6f56b568975b0b97cb897ce5fff15";

        Ok(checksum != correct_checksum)
    }

    fn check_storage_consistency(&self, register: &PackageRegister) -> Result<Option<Issue>, VerifierError> {
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

    fn package_storage_is_consistent(&self, package_id: &PackageId) -> Result<bool, VerifierError> {
        let installed_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Check if the directory exists, if so return true
        if fs::exists(installed_directory)? {
            return Ok(true);
        }

        Ok(false)
    }

    fn check_register_consistency(&self, register: &PackageRegister) -> Result<Option<Issue>, VerifierError> {
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
                let package_id = PackageId::new(package_name, &version);

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

    fn register_package_is_consistent(&self, package_id: &PackageId, register: &PackageRegister) -> Result<bool, VerifierError> {
        let install_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Return no issue if the package exists in storage
        if !install_directory.is_dir() {
            return Ok(true);
        }

        // Return no issue if the package exists in the register
        if register.get_package_version(package_id).is_some() {
            return Ok(true);
        }

        // Return an inconsistent register issue if the package exists in storage, but not in the register
        Ok(false)
    }

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

    pub fn issues_found(&self) -> bool {
        self.issues_found
    }
}
