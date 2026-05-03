// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, path::PathBuf, str::FromStr};

use crate::{
    cli::display::logging::{debug, warning},
    config::{Config, Repository},
    installer::types::{PackageId, PackageName, Version},
    packager,
    platforms::Target,
    repositories::{provider, types::Checksum},
    storage::package_register::PackageRegister,
    verifier::{
        Issue,
        error::{Result, VerifierError},
    },
};

#[derive(Debug, PartialEq, Eq, Hash)]
enum Check {
    // Initial checks (checks which verify methods which the verifier uses internally)
    ConfigExistance,

    // General package checks
    StorageConsistency,
    RegisterConsistency,
    DependencyTree,
    Alterations,
    PackitGroup,

    // Checks which are specific to a package
    PackageExistance,
    PackageStorageConsistancy,
    PackageRegisterConsistency,
    PackageDependencyTree,
    PackageAlterations,
}

impl Check {
    /// Gets the dependencies of a check (the checks which should happen before the given check).
    fn get_dependencies(&self) -> &[Self] {
        match self {
            // Initial checks
            Self::ConfigExistance => &[],

            // General checks
            Self::PackitGroup => &[],
            Self::StorageConsistency => &[Self::PackitGroup],
            Self::RegisterConsistency => &[Self::PackitGroup],
            Self::DependencyTree => &[Self::PackitGroup, Self::StorageConsistency, Self::RegisterConsistency],
            Self::Alterations => &[Self::PackitGroup, Self::StorageConsistency, Self::RegisterConsistency],

            // Package specific checks
            Self::PackageExistance => &[],
            Self::PackageStorageConsistancy => &[Self::PackageExistance],
            Self::PackageRegisterConsistency => &[Self::PackageExistance],
            Self::PackageDependencyTree => &[Self::PackageExistance, Self::PackageStorageConsistancy, Self::PackageDependencyTree],
            Self::PackageAlterations => &[Self::PackageExistance, Self::PackageStorageConsistancy, Self::PackageDependencyTree],
        }
    }

    /// Gets all intial checks.
    fn get_initial_checks<'a>() -> &'a [Self] {
        &[Self::ConfigExistance]
    }

    /// Gets all general checks.
    fn get_general_checks<'a>() -> &'a [Self] {
        &[
            Self::StorageConsistency,
            Self::RegisterConsistency,
            Self::DependencyTree,
            Self::Alterations,
            Self::PackitGroup,
        ]
    }

    /// Gets all package specific checks.
    fn get_package_checks<'a>() -> &'a [Self] {
        &[
            Self::PackageExistance,
            Self::PackageStorageConsistancy,
            Self::PackageRegisterConsistency,
            Self::PackageDependencyTree,
            Self::PackageAlterations,
        ]
    }

    /// Gets the checks in the correct order based on the 'check dependency tree'.
    /// Returns a flattened 'check dependency tree'
    fn get_ordered_checks<'a>(checks: &'a [Self]) -> Vec<&'a Self> {
        let mut ordered = Vec::new();
        for check in checks {
            ordered.extend(Self::get_ordered_checks(check.get_dependencies()));
            ordered.push(check);
        }

        let mut seen = HashSet::new();
        let mut unique_ordered = Vec::new();
        for check in ordered {
            if !seen.contains(check) {
                unique_ordered.push(check);
                seen.insert(check);
            }
        }

        unique_ordered
    }
}

/// Verifier that scans the Packit environment for issues.
pub struct Verifier {
    current_intial_check: usize,
    current_general_check: usize,
    current_package_check: usize,
    issues_found: bool,
}

impl Verifier {
    /// Creates a new verifier.
    pub fn new() -> Self {
        Self {
            current_intial_check: 0,
            current_general_check: 0,
            current_package_check: 0,
            issues_found: false,
        }
    }

    /// A wrapper around the `next_initial_issue_impl` method.
    /// Returns the next issue if it exists.
    /// Returns an error if an error is returned and no issues have been found yet.
    pub fn next_initial_issue(&mut self) -> Result<Option<Issue>> {
        match self.next_initial_issue_impl() {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining issues.");
                self.current_intial_check = Check::get_initial_checks().len();
                return Ok(None);
            },
            Err(e) => return Err(e),
        }
    }

    /// Gets the next initial issue.
    /// Returns None if there are no issues to return.
    fn next_initial_issue_impl(&mut self) -> Result<Option<Issue>> {
        loop {
            let ordered_checks = Check::get_ordered_checks(Check::get_initial_checks());
            let check = match ordered_checks.get(self.current_intial_check) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase current issue
            self.current_intial_check += 1;

            let issue = match check {
                Check::ConfigExistance => self.check_config_existance()?,

                // Make sure that the check is not an initial check
                _ if Check::get_initial_checks().contains(check) => panic!("TODO"),

                // Continue if the check is not an initial check
                _ => continue,
            };

            if let Some(issue) = issue {
                self.issues_found = true;
                return Ok(Some(issue));
            }
        }
    }

    /// A wrapper around the `next_package_issue_impl` method.
    /// Returns the next issue if it exists.
    /// Returns an error if an error is returned and no issues have been found yet.
    pub fn next_issue(&mut self, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        // Make sure the initial checks have been run before doing general checks
        if self.current_intial_check != Check::get_initial_checks().len() {
            return Err(VerifierError::InitialChecksSkippedError);
        }

        match self.next_issue_impl(register, config) {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining issues.");
                self.current_general_check = Check::get_general_checks().len();
                return Ok(None);
            },
            Err(e) => return Err(e),
        }
    }

    /// Gets the next general issue.
    /// Returns None if there are no issues to return.
    fn next_issue_impl(&mut self, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        loop {
            let ordered_checks = Check::get_ordered_checks(Check::get_general_checks());
            let check = match ordered_checks.get(self.current_general_check) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase current issue
            self.current_general_check += 1;

            let issue = match check {
                Check::StorageConsistency => self.check_storage_consistency(register, config)?,
                Check::RegisterConsistency => self.check_register_consistency(register, config)?,
                Check::DependencyTree => self.check_dependency_tree(register),
                Check::Alterations => self.check_alterations(register, config)?,
                Check::PackitGroup => self.check_packit_group(config),

                // Make sure that the check is not a general check
                _ if Check::get_general_checks().contains(check) => panic!("TODO"),

                // Continue if the check is package specific or is an initial check
                _ => continue,
            };

            if let Some(issue) = issue {
                self.issues_found = true;
                return Ok(Some(issue));
            }
        }
    }

    /// A wrapper around the `next_package_issue_impl` method.
    /// Returns the next package issue if it exists.
    /// Returns an error if an error is returned and no issues have been found yet.
    pub fn next_package_issue(&mut self, package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        match self.next_package_issue_impl(package_id, register, config) {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining issues.");
                self.current_package_check = Check::get_package_checks().len();
                return Ok(None);
            },
            Err(e) => return Err(e),
        }
    }

    /// Gets the next issue for a specific package in the order defined in VERIFY_ORDER.
    /// Returns None if there are no more issues to return.
    fn next_package_issue_impl(&mut self, package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        loop {
            let ordered_checks = Check::get_ordered_checks(Check::get_package_checks());
            let check = match ordered_checks.get(self.current_package_check) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase current issue
            self.current_package_check += 1;

            let issue = match check {
                Check::PackageExistance if self.check_package_existance(package_id, register, config)? => continue,
                Check::PackageExistance => Issue::NotFound(package_id.clone()),
                Check::StorageConsistency if self.package_storage_is_consistent(package_id, config)? => continue,
                Check::StorageConsistency => Issue::InconsistentStorage(vec![package_id.clone()]),
                Check::RegisterConsistency if self.register_package_is_consistent(package_id, register) => continue,
                Check::RegisterConsistency => Issue::InconsistentRegister(vec![package_id.clone()]),
                Check::DependencyTree => match self.check_package_dependency_tree(package_id, register) {
                    Some(issue) => issue,
                    None => continue,
                },
                Check::Alterations if !self.check_package_alterations(package_id, register, config)? => continue,
                Check::Alterations => Issue::AlteredPackage(vec![package_id.clone()]),

                // Make sure that the check is not a package specific check
                _ if Check::get_package_checks().contains(check) => panic!("TODO"),

                // Continue if the check is not package specific
                _ => continue,
            };

            self.issues_found = true;
            return Ok(Some(issue));
        }
    }

    /// Checks for alterations in all packages using a checksum which is compared to the checksum from the pre-build.
    /// Returns an alteration issue or None if no packages can be found that are altered.
    #[expect(unused_variables)]
    fn check_alterations(&self, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        // TODO: For now skip this check, because it will never work (yet)
        return Ok(None);
        warning!("This is an experimental check, issues from this check could be inaccurate.");

        // Check issue for all installed packages
        let mut altered = Vec::new();
        for package in register.iterate_all() {
            if self.check_package_alterations(&package.package_id, register, config)? {
                altered.push(package.package_id.clone());
            }
        }

        if altered.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::AlteredPackage(altered)))
    }

    /// Checks if the config exists.
    /// Returns `None` if the config exists or an `Issue::MissingConfig` otherwise.
    /// Could return an IO error.
    fn check_config_existance(&self) -> Result<Option<Issue>> {
        if fs::exists(&Config::get_default_path())? {
            return Ok(None);
        }

        Ok(Some(Issue::MissingConfig))
    }

    /// Checks for alterations in a single package using a checksum which is compared to the checksum from the pre-build.
    /// Returns true if the package was altered, false if not.
    #[expect(unused_variables, unreachable_code)]
    fn check_package_alterations(&self, package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<bool> {
        // TODO: For now skip this check, because it will never work (yet)
        return Ok(false);

        // Get the installed package from the register
        let Some(package_version) = register.get_package_version(package_id) else {
            warning!("Cannot retrieve package '{package_id}' from register for package alterations check, skipping check");
            return Ok(false);
        };

        let mut prebuilds_url = package_version.source_prebuild_repository_url.clone();
        let mut prebuilds_provider = package_version.source_prebuild_repository_provider.clone();

        if prebuilds_url.is_none() {
            let repository = Repository::new(&package_version.source_repository_url, &package_version.source_repository_provider);

            let Some(provider) = provider::create_metadata_provider(&repository) else {
                warning!("Cannot create metadata provider for '{package_id}', skipping check");
                return Ok(false);
            };

            let repo_metadata = match provider.read_repository_metadata() {
                Ok(meta) => meta,
                Err(e) => {
                    warning!("Cannot retrieve repository metadata for '{package_id}', skipping check");
                    debug!(err: e, "Retrieving repository metadata failed");
                    return Ok(false);
                },
            };

            prebuilds_url = repo_metadata.prebuilds_url;
            prebuilds_provider = repo_metadata.prebuilds_provider;
        }

        let Some(prebuilds_url) = &prebuilds_url else {
            warning!(
                "Cannot perform alterations check for package '{package_id}', because no prebuild repository for the package can be found, skipping check"
            );
            return Ok(false);
        };

        let provider = match provider::create_prebuild_provider_from_url(prebuilds_url, prebuilds_provider) {
            Some(provider) => provider,
            None => {
                warning!("Cannot create prebuild provider for '{package_id}', skipping check");
                return Ok(false);
            },
        };

        let revision = package_version.revisions.len() as u64;
        let correct_checksum = match provider.get_prebuild_checksum(package_id, revision, &Target::current()) {
            Ok(Some(checksum)) => checksum,
            Ok(None) => {
                warning!(
                    "Cannot perform alterations check for package '{package_id}', because no checksum of the prebuild can be found, skipping check"
                );
                return Ok(false);
            },
            Err(e) => {
                warning!("Cannot perform alterations check for package '{package_id}', because checksum cannot be read, skipping check");
                debug!(err: e, "Failed to read prebuild checksum");
                return Ok(false);
            },
        };

        let install_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
        let compressed = packager::compress(&install_directory)?;
        let checksum = Checksum::from_bytes(&compressed);

        Ok(checksum != correct_checksum)
    }

    /// Checks if a package exists in the register or in storage.
    /// Returns true if the package exists, false if not.
    /// Could return an IO error.
    fn check_package_existance(&self, package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<bool> {
        let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
        if register.get_package_version(package_id).is_none() && !fs::exists(installed_directory)? {
            return Ok(false);
        }

        Ok(true)
    }

    /// Checks if all packages in the register also exist in the package storage in the prefix directory.
    /// Returns a storage consistency issue or None if there are no packages missing from storage.
    fn check_storage_consistency(&self, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        let mut missing = Vec::new();
        for package in register.iterate_all() {
            if !self.package_storage_is_consistent(&package.package_id, config)? {
                missing.push(package.package_id.clone());
            }
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InconsistentStorage(missing)))
    }

    /// Checks if a specific package exists in storage. Note that it doesn't check if the package also exists in the register.
    /// Returns false if the package can not be found in the storage, true if it can be found.
    fn package_storage_is_consistent(&self, package_id: &PackageId, config: &Config) -> Result<bool> {
        let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Check if the directory exists, if so return true
        if fs::exists(&installed_directory)? && !self.directory_is_empty(&installed_directory)? {
            return Ok(true);
        }

        Ok(false)
    }

    /// Checks recursively if a directory is empty (contains nothing but empty directories).
    /// Returns true if empty, false if not.
    fn directory_is_empty(&self, directory: &PathBuf) -> Result<bool> {
        for package in directory.read_dir()? {
            let package = package?;

            if !package.path().is_dir() {
                return Ok(false);
            }

            if !self.directory_is_empty(&package.path())? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Checks if all packages in storage also exist in the register.
    /// Returns a register consistency issue or None if there are packages missing from the register.
    fn check_register_consistency(&self, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        let package_directory = config.prefix_directory.join("packages");
        let mut missing = Vec::new();
        for file_package in fs::read_dir(package_directory)? {
            let file_package = file_package?;
            if !file_package.path().is_dir() {
                continue;
            }

            // Get the package name
            let package_name = file_package.file_name();
            let package_name = package_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?;
            let package_name = PackageName::from_str(package_name)?;

            for file_version in fs::read_dir(file_package.path())? {
                let file_version = file_version?;
                if !file_version.path().is_dir() {
                    continue;
                }

                // Get the version, and create the package id
                let version = Version::from_str(file_version.file_name().to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;
                let package_id = PackageId::new(package_name.clone(), version);

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
    fn register_package_is_consistent(&self, package_id: &PackageId, register: &PackageRegister) -> bool {
        // Return no issue if the package exists in the register
        if register.get_package_version(package_id).is_some() {
            return true;
        }

        // Return an inconsistent register issue if the package exists in storage, but not in the register
        false
    }

    /// Checks the completeness of the depedency trees from the packages.
    /// Returns a dependency tree issue or None if there are no packages missing from the dependency trees.
    fn check_dependency_tree(&self, register: &PackageRegister) -> Option<Issue> {
        let mut all_missing = Vec::new();
        for package in register.iterate_all() {
            all_missing.extend(self.check_package_dependency_tree_impl(&package.package_id, register));
        }

        if all_missing.is_empty() {
            return None;
        }

        Some(Issue::BrokenTree(all_missing))
    }

    /// Wraps around the `check_package_dependency_tree_impl` method to convert it into an `Option<Issue>`.
    /// Returns an `Issue::BrokenTree` if missing packages are found, None if not packages are missing.
    fn check_package_dependency_tree(&self, package_id: &PackageId, register: &PackageRegister) -> Option<Issue> {
        let packages = self.check_package_dependency_tree_impl(package_id, register);
        if packages.is_empty() {
            return None;
        }

        return Some(Issue::BrokenTree(packages));
    }

    /// Checks the completeness of the dependency tree from a specific package.
    /// Returns a list of missing packages, can be empty if there are no packages missing from the tree.
    fn check_package_dependency_tree_impl(&self, package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
        let package = match register.get_package_version(package_id) {
            Some(package) => package,
            None => {
                debug!("Parent node '{package_id}' doesn't exist, while checking dependency tree.");
                return Vec::new();
            },
        };

        let mut missing = Vec::new();
        for dependency in &package.dependencies {
            if register.get_package_version(dependency).is_none() {
                missing.push((package_id.clone(), dependency.clone()));
                continue;
            }

            // Recursively check if all the dependencies are installed
            missing.extend(self.check_package_dependency_tree_impl(dependency, register));
        }

        missing
    }

    /// Checks if the packit group exists if multiuser mode is enabled in the config.
    /// Returns the issue if the group does not exist, None otherwise.
    fn check_packit_group(&self, config: &Config) -> Option<Issue> {
        if !config.multiuser {
            return None; // We don't need the packit group if multiuser mode is not enabled
        }

        // TODO: Also implement for Windows
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            use crate::platforms::permissions;

            if permissions::platform::get_group_id(permissions::PACKIT_GROUP_NAME).is_err() {
                return Some(Issue::MissingPackitGroup);
            }
        }

        None
    }

    /// Get the issues found states.
    /// Returns true if issues are found, false otherwise.
    pub fn issues_found(&self) -> bool {
        self.issues_found
    }

    /// Reverses the initial checks counter by 1. Except if the current is 0.
    pub fn reverse_initial_check(&mut self) {
        if self.current_intial_check == 0 {
            return;
        }

        self.current_intial_check -= 1;
    }

    /// Reverses the general checks counter by 1. Except if the current is 0.
    pub fn reverse_general_check(&mut self) {
        if self.current_general_check == 0 {
            return;
        }

        self.current_general_check -= 1;
    }

    /// Reverses the package checks counter by 1. Except if the current is 0.
    pub fn reverse_package_check(&mut self) {
        if self.current_package_check == 0 {
            return;
        }

        self.current_package_check -= 1;
    }
}
