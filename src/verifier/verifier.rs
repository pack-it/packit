// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    cli::display::logging::{debug, warning},
    config::{Config, Repository},
    installer::types::{Dependency, PackageId, PackageName, Version},
    packager,
    platforms::{
        DEFAULT_PREFIX, Target,
        permissions::{does_packit_group_exist, is_writable},
    },
    register::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    repositories::{
        provider::{self, create_metadata_provider},
        types::{Checksum, PackageVersionMeta},
    },
    utils::io::directory_is_empty,
    verifier::{
        Issue,
        checks::Check,
        error::{Result, VerifierError},
        utils::get_storage_packages,
    },
};

/// Verifier that scans the Packit environment for issues.
pub struct Verifier {
    current_intial_check: usize,
    current_check: usize,
    issues_found: bool,
}

impl Verifier {
    /// Creates a new verifier.
    pub fn new() -> Self {
        Self {
            current_intial_check: 0,
            current_check: 0,
            issues_found: false,
        }
    }

    /// Gets the next initial issue if it exists.
    /// If an error occurs during the check it's only returned if no previous issues were found.
    pub fn next_initial_issue(&mut self) -> Result<Option<Issue>> {
        match self.next_initial_issue_impl() {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining issues.");
                self.current_intial_check = Check::get_initial_checks().len();
                Ok(None)
            },
            Err(e) => Err(e),
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
                Check::Permissions => self.check_permissions()?,
                Check::ConfigExistence => self.check_config_existence()?,
                Check::ConfigSyntax => self.check_config_syntax()?,
                Check::RegisterExistence => self.check_register_existence()?,
                Check::RegisterSyntax => self.check_register_syntax()?,

                // Make sure that the check is not an initial check
                _ if Check::get_initial_checks().contains(check) => return Err(VerifierError::UnimplementedCheckError),

                // Continue if the check is not an initial check
                _ => continue,
            };

            if let Some(issue) = issue {
                self.issues_found = true;
                return Ok(Some(issue));
            }
        }
    }

    /// Gets the next general issue if it exists.
    /// If an error occurs during the check it's only returned if no previous issues were found.
    pub fn next_issue(&mut self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        // Make sure the initial checks have been run before doing general checks
        if self.current_intial_check != Check::get_initial_checks().len() {
            return Err(VerifierError::InitialChecksSkippedError);
        }

        match self.next_issue_impl(packages, register, config) {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining issues.");
                self.current_check = Check::get_checks().len();
                Ok(None)
            },
            Err(e) => Err(e),
        }
    }

    /// Gets the next general issue.
    /// Returns None if there are no issues to return.
    fn next_issue_impl(&mut self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        loop {
            let ordered_checks = Check::get_ordered_checks(Check::get_checks());
            let check = match ordered_checks.get(self.current_check) {
                Some(check) => check,
                None => return Ok(None),
            };

            // Increase current issue
            self.current_check += 1;

            let issue = match check {
                Check::PackitGroup => self.check_packit_group(config)?,
                Check::StrayDirectory => self.check_stray_directories(config)?,
                Check::RegisterConsistency => self.check_register_consistency(register, config)?, // TODO: Check
                Check::StorageConsistency => self.check_storage_consistency(packages, config)?,
                Check::DependencyTree => self.check_dependency_tree(packages, register),
                Check::Alterations => self.check_alterations(packages, register, config)?,
                Check::MissingDependencies => self.check_missing_dependencies(packages, register)?,
                Check::InvalidDependencies => self.check_invalid_dependencies(packages, register)?,
                Check::MissingDependents => self.check_missing_dependents(packages, register),
                Check::InvalidDependents => self.check_invalid_dependents(packages, register),
                Check::InvalidActive => self.check_invalid_active(&packages.iter().map(|p| p.name.clone()).collect(), register, config)?,
                Check::ForbiddenLink => self.check_forbidden_link(packages, register)?,
                Check::MissingLink => self.check_missing_link(packages, register, config)?,

                // Make sure that the check is not a general check
                _ if Check::get_checks().contains(check) => return Err(VerifierError::UnimplementedCheckError),

                // Continue if the check is package specific or is an initial check
                _ => continue,
            };

            if let Some(issue) = issue {
                self.issues_found = true;
                return Ok(Some(issue));
            }
        }
    }

    /// Checks the permissions of the prefix directory and all its sub directories.
    /// If the config can be used it will use the prefix directory specified there,
    /// otherwise the default prefix directory is checked.
    fn check_permissions(&self) -> Result<Option<Issue>> {
        let prefix_directory = match Config::from(&Config::get_default_path()) {
            Ok(config) => config.prefix_directory,
            Err(_) => DEFAULT_PREFIX.into(),
        };

        let unwritable = self.check_permissions_impl(&prefix_directory)?;
        if unwritable.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::IncorrectPermissions(unwritable.into_iter().collect())))
    }

    /// Recursively checks if all files in directory are writable.
    /// Returns all directories which are not writable (could be empty).
    fn check_permissions_impl(&self, directory: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut unwritable = Vec::new();
        if !is_writable(directory)? {
            unwritable.push(directory.clone());
        }

        if !directory.is_dir() {
            return Ok(unwritable);
        }

        // Recurse
        for sub_directory in fs::read_dir(directory)? {
            let sub_directory = sub_directory?;
            unwritable.extend(self.check_permissions_impl(&sub_directory.path())?);
        }

        Ok(unwritable)
    }

    /// Checks if the Config.toml exists.
    /// Returns `None` if the config exists or an `Issue::MissingConfig` otherwise.
    /// Could return an IO error.
    fn check_config_existence(&self) -> Result<Option<Issue>> {
        if fs::exists(Config::get_default_path())? {
            return Ok(None);
        }

        Ok(Some(Issue::MissingConfig))
    }

    /// Checks if the `Config.toml` syntax is valid.
    /// Returns `None` if the config syntax is valid or an `Issue::MissingConfig` otherwise.
    /// Could return an IO error.
    fn check_config_syntax(&self) -> Result<Option<Issue>> {
        // Don't return a config syntax issue if the `Config.toml` doesn't exist
        if !fs::exists(Config::get_default_path())? {
            return Ok(None);
        }

        match Config::from(&Config::get_default_path()) {
            Ok(_) => Ok(None),
            Err(_) => Ok(Some(Issue::MissingConfig)),
        }
    }

    /// Checks if the Register.toml exists.
    /// Returns `None` if the register exists or an `Issue::MissingRegister` otherwise.
    fn check_register_existence(&self) -> Result<Option<Issue>> {
        let config = Config::from(&Config::get_default_path())?;
        let register_directory = &PackageRegister::get_path(&config.prefix_directory);
        if fs::exists(register_directory)? {
            return Ok(None);
        }

        Ok(Some(Issue::MissingRegister))
    }

    /// Checks if the Register.toml syntax is valid.
    /// Returns `None` if the register syntax is valid or an `Issue::MissingRegister` otherwise.
    fn check_register_syntax(&self) -> Result<Option<Issue>> {
        let config = Config::from(&Config::get_default_path())?;
        let register_directory = &PackageRegister::get_path(&config.prefix_directory);

        // Don't return a register syntax issue if the `Register.toml` doesn't exist
        if !fs::exists(register_directory)? {
            return Ok(None);
        }

        match PackageRegister::from(register_directory) {
            Ok(_) => Ok(None),
            Err(_) => Ok(Some(Issue::MissingRegister)),
        }
    }

    /// Checks for alterations in all packages using a checksum which is compared to the checksum from the pre-build.
    /// Returns an alteration issue or None if no packages can be found that are altered.
    #[expect(unused_variables, unreachable_code)]
    fn check_alterations(&self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        // TODO: For now skip this check, because it will never work (yet)
        return Ok(None);
        warning!("This is an experimental check, issues from this check could be inaccurate.");

        // Check issue for all installed packages
        let mut altered = Vec::new();
        for package_id in packages {
            if self.check_package_alterations(package_id, register, config)? {
                altered.push(package_id.clone());
            }
        }

        if altered.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::AlteredPackage(altered)))
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

        let mut prebuilds_url = package_version.prebuilds_repository_url.clone();
        let mut prebuilds_provider = package_version.prebuilds_repository_provider.clone();

        if prebuilds_url.is_none() {
            let repository = Repository::new(
                &package_version.metadata_repository_url,
                &package_version.metadata_repository_provider,
            );

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

    /// Checks if all packages in the register also exist in the package storage in the prefix directory.
    /// Returns a storage consistency issue or None if there are no packages missing from storage.
    fn check_storage_consistency(&self, packages: &Vec<PackageId>, config: &Config) -> Result<Option<Issue>> {
        let mut missing = Vec::new();
        for package_id in packages {
            if !self.package_storage_is_consistent(&package_id, config)? {
                missing.push(package_id.clone());
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
        if fs::exists(&installed_directory)? && !directory_is_empty(&installed_directory)? {
            return Ok(true);
        }

        Ok(false)
    }

    /// Checks if all packages in storage also exist in the register.
    /// Returns a register consistency issue or None if there are packages missing from the register.
    fn check_register_consistency(&self, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        let storage_packages = get_storage_packages(config)?;
        let mut missing = HashSet::new();
        for package_id in storage_packages {
            // Check if the package version also exists in the register, if not add it to missing
            if register.get_package_version(&package_id).is_none() {
                missing.insert(package_id);
            }
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InconsistentRegister(missing)))
    }

    /// Checks for all packages if the package active is valid.
    fn check_invalid_active(&self, packages: &HashSet<PackageName>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        let mut invalid_active = Vec::new();
        for package_name in packages {
            if let Some(package) = self.check_invalid_package_active(package_name, register, config)? {
                invalid_active.push(package);
            }
        }

        if invalid_active.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InvalidActive(invalid_active)))
    }

    /// Checks for a given package name if the package active is valid.
    /// It's invalid if:
    ///     - the link or link destination doesn't exist
    ///     - if the package version doesn't exist
    ///     - if the version specified in the register doesn't match the linked version
    fn check_invalid_package_active(
        &self,
        package_name: &PackageName,
        register: &PackageRegister,
        config: &Config,
    ) -> Result<Option<PackageName>> {
        let Some(package) = register.get_package(package_name) else {
            return Ok(None);
        };

        let active_directory = config.prefix_directory.join("active").join(package_name);
        if !fs::exists(&active_directory)? {
            return Ok(Some(package_name.clone()));
        }

        let active_link = fs::read_link(&active_directory)?;
        if active_link != config.prefix_directory.join("packages").join(package_name).join(package.active_version.to_string()) {
            return Ok(Some(package_name.clone()));
        }

        Ok(None)
    }

    /// Checks all packages for a forbidden link. Where a forbidden link is a package which is symlinked
    /// while it shouldn't be according to the repository metadata.
    fn check_forbidden_link(&self, packages: &Vec<PackageId>, register: &PackageRegister) -> Result<Option<Issue>> {
        let mut forbidden = Vec::new();
        for package_id in packages {
            forbidden.extend(self.check_forbidden_package_link(package_id, register)?);
        }

        if forbidden.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::ForbiddenLink(forbidden)))
    }

    /// Checks a given package for a forbidden link. Where a forbidden link is a package which is symlinked
    /// while it shouldn't be according to the repository metadata.
    fn check_forbidden_package_link(&self, package_id: &PackageId, register: &PackageRegister) -> Result<Vec<PackageName>> {
        let Some(package_version) = register.get_package_version(package_id) else {
            return Ok(Vec::new());
        };

        // Check if this package is allowed to be symlinked
        // Assume the package version meta can be found (otherwise no issue is returned)
        let link_allowed = match self.get_package_version_meta(package_id, package_version)? {
            Some(package_version) => {
                let package_target = package_version.get_target(&package_version.get_best_target(&Target::current())?)?;
                match package_target.skip_symlinking {
                    Some(skip_symlinking) => !skip_symlinking,
                    None => !package_version.skip_symlinking,
                }
            },
            None => true,
        };

        // Return early if symlinking is allowed according to the metadata
        if link_allowed {
            return Ok(Vec::new());
        }

        let Some(package) = register.get_package(&package_id.name) else {
            return Ok(Vec::new());
        };

        if !package.symlinked {
            return Ok(Vec::new());
        }

        Ok(vec![package_id.name.clone()])
    }

    /// Checks for all packages if symlinks are missing.
    /// Returns an `Issue::MissingLinks` if symlinks are missing, None otherwise.
    fn check_missing_link(&self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        let mut missing = Vec::new();
        for package_id in packages {
            if self.check_missing_package_link(package_id, register, config)? {
                missing.push(package_id.name.clone());
            }
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::MissingLinks(missing)))
    }

    /// Checks for a given package if symlinks are missing.
    /// Returns false if the given package is not the active package or if the package shouldn't be symlinked.
    /// Returns true if symlinks are missing, false otherwise.
    fn check_missing_package_link(&self, package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<bool> {
        let Some(package) = register.get_package(&package_id.name) else {
            return Ok(false);
        };

        // Return early if the package is not the active version or if the packages shouldn't be symlinked
        if package.active_version != package_id.version || !package.symlinked {
            return Ok(false);
        }

        let package_path = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
        for directory_name in ["bin", "include", "lib", "share"] {
            let symlink_directory = config.prefix_directory.join(directory_name);
            let directory = package_path.join(directory_name);

            // Continue if the directory doesn't exist in the package
            if !fs::exists(&directory)? {
                continue;
            }

            if self.check_symlinks(&directory, &symlink_directory)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Checks if a symlink can be found for the given directory.
    /// Returns true if a symlink cannot be found, false otherwise.
    fn check_symlinks(&self, directory: &PathBuf, symlink_directory: &Path) -> Result<bool> {
        for file in fs::read_dir(directory)? {
            let file = file?;
            let file_path = file.path();

            // Recurse
            if file_path.is_dir() {
                if self.check_symlinks(&file_path, &symlink_directory.join(file.file_name()))? {
                    return Ok(true);
                }

                continue;
            }

            // Read the sylink metadata to check if the symlink exists
            if fs::symlink_metadata(symlink_directory.join(file.file_name())).is_err() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Checks for all packages if they have invalid dependents. Where an invalid dependent is a package which doesn't
    /// exist or a package which doesn't have the given package as a dependency.
    /// Returns a `Vec<(PackageId, PackageId)>`, with format `<child> : <dependent>`.
    fn check_invalid_dependents(&self, packages: &Vec<PackageId>, register: &PackageRegister) -> Option<Issue> {
        let mut invalid_dependents = Vec::new();
        for package_id in packages {
            invalid_dependents.extend(self.check_invalid_package_dependents(package_id, register));
        }

        if invalid_dependents.is_empty() {
            return None;
        }

        Some(Issue::InvalidDependents(invalid_dependents))
    }

    /// Checks if a given package has invalid dependents. Where an invalid dependent is a package which doesn't
    /// exist or a package which doesn't have the given package as a dependency.
    /// Returns a `Vec<(PackageId, PackageId)>`, with format `<child> : <dependent>`.
    fn check_invalid_package_dependents(&self, package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
        let mut invalid_dependents = Vec::new();
        let Some(package_version) = register.get_package_version(package_id) else {
            return invalid_dependents;
        };

        for dependent in &package_version.dependents {
            match register.get_package_version(dependent) {
                Some(package) if !package.dependencies.contains(package_id) => {
                    invalid_dependents.push((package_id.clone(), dependent.clone()));
                },
                Some(_) => {},
                None => invalid_dependents.push((package_id.clone(), dependent.clone())),
            }
        }

        invalid_dependents
    }

    /// Checks for all packages if they miss any of their dependents.
    fn check_missing_dependents(&self, packages: &Vec<PackageId>, register: &PackageRegister) -> Option<Issue> {
        let mut missing_dependents = Vec::new();
        for package_id in packages {
            missing_dependents.extend(self.check_missing_package_dependents(package_id, register));
        }

        if missing_dependents.is_empty() {
            return None;
        }

        Some(Issue::MissingDependents(missing_dependents))
    }

    /// Check for the given package if its dependencies have it as a dependent.
    fn check_missing_package_dependents(&self, package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
        let mut missing_dependents = Vec::new();
        let Some(package_version) = register.get_package_version(package_id) else {
            return Vec::new();
        };

        for package in register.iterate_all() {
            // Make sure not to check itself
            if package.package_id == *package_id {
                continue;
            }

            // Missing dependents
            if package.dependencies.contains(package_id) && !package_version.dependents.contains(&package.package_id) {
                missing_dependents.push((package_id.clone(), package.package_id.clone()));
            }
        }

        missing_dependents
    }

    /// Checks for missing dependencies in the given packages.
    /// Returns an `Issue::MissingDependencies` with the missing dependencies, or `None` if no dependencies are missing.
    fn check_missing_dependencies(&self, packages: &Vec<PackageId>, register: &PackageRegister) -> Result<Option<Issue>> {
        let mut missing = Vec::new();
        for package_id in packages {
            missing.extend(self.check_missing_package_dependencies(package_id, register)?);
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::MissingDependencies(missing)))
    }

    /// Checks for missing dependencies for a single package.
    /// Returns an `Issue::MissingDependencies` with the missing dependencies, or `None` if no dependencies are missing.
    /// TODO: Unnecessary function
    fn check_missing_package_dependencies(
        &self,
        package_id: &PackageId,
        register: &PackageRegister,
    ) -> Result<Vec<(PackageId, Dependency)>> {
        let Some(package) = register.get_package_version(package_id) else {
            return Ok(Vec::new());
        };

        self.missing_dependencies_impl(package)
    }

    /// Checks if a given package misses dependencies in the register according to the repository metadata.
    /// Returns a list of missing dependencies for the given package (can be empty).
    /// TODO: Pass a mutable reference to missing (also in some other places)
    fn missing_dependencies_impl(&self, package: &InstalledPackageVersion) -> Result<Vec<(PackageId, Dependency)>> {
        let package_id = &package.package_id;
        let mut missing = Vec::new();

        // Assume the package version meta can be found (otherwise no issue is returned)
        let Some(package_version_meta) = self.get_package_version_meta(package_id, package)? else {
            return Ok(Vec::new());
        };

        // Check if each dependency is satisfied
        for metadata_dependency in package_version_meta.dependencies {
            let mut satisfied = false;
            for dependency in &package.dependencies {
                if metadata_dependency.satisfied(&dependency.name, &dependency.version) {
                    satisfied = true;
                }
            }

            if !satisfied {
                missing.push((package_id.clone(), metadata_dependency));
            }
        }

        Ok(missing)
    }

    /// Checks for invalid dependencies in all packages.
    /// Returns an `Issue::InvalidDependencies` with the invalid dependencies, or `None` if no dependencies are invalid.
    fn check_invalid_dependencies(&self, packages: &Vec<PackageId>, register: &PackageRegister) -> Result<Option<Issue>> {
        let mut invalid = Vec::new();
        for package_id in packages {
            invalid.extend(self.check_invalid_package_dependencies(package_id, register)?);
        }

        if invalid.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InvalidDependencies(invalid)))
    }

    /// Checks for invalid dependencies for a single package.
    /// Returns an `Issue::InvalidDependencies` with the invalid dependencies, or `None` if no dependencies are invalid.
    /// TODO: Unnecessary function
    fn check_invalid_package_dependencies(
        &self,
        package_id: &PackageId,
        register: &PackageRegister,
    ) -> Result<Vec<(PackageId, PackageId)>> {
        let Some(package) = register.get_package_version(package_id) else {
            return Ok(Vec::new());
        };

        self.invalid_dependencies_impl(package)
    }

    /// Checks if a given package has invalid dependencies in the register according to the repository metadata.
    /// Returns a list of invalid dependencies for the given package (can be empty).
    /// TODO: Pass invalid as mutable ref
    fn invalid_dependencies_impl(&self, package: &InstalledPackageVersion) -> Result<Vec<(PackageId, PackageId)>> {
        let package_id = &package.package_id;
        let mut invalid = Vec::new();

        // Assume the package version meta can be found (otherwise no issue is returned)
        let Some(package_version_meta) = self.get_package_version_meta(package_id, package)? else {
            return Ok(Vec::new());
        };

        // Check if there is a package dependency which doesn't satisfy any of the metadata dependencies
        for dependency in &package.dependencies {
            let mut satisfied = false;
            for metadata_dependency in &package_version_meta.dependencies {
                if metadata_dependency.satisfied(&dependency.name, &dependency.version) {
                    satisfied = true;
                }
            }

            if !satisfied {
                invalid.push((package_id.clone(), dependency.clone()));
            }
        }

        Ok(invalid)
    }

    /// Checks the completeness of the depedency trees from the packages.
    /// Returns a dependency tree issue or None if there are no packages missing from the dependency trees.
    fn check_dependency_tree(&self, packages: &Vec<PackageId>, register: &PackageRegister) -> Option<Issue> {
        let mut all_missing = Vec::new();
        for package_id in packages {
            all_missing.extend(self.check_package_dependency_tree(package_id, register));
        }

        if all_missing.is_empty() {
            return None;
        }

        Some(Issue::BrokenTree(all_missing))
    }

    /// Wraps around the `check_package_dependency_tree_impl` method to convert it into an `Option<Issue>`.
    /// Returns an `Issue::BrokenTree` if missing packages are found, None if not packages are missing.
    /// TODO: Unnecessary function
    fn check_package_dependency_tree(&self, package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
        self.check_package_dependency_tree_impl(package_id, register)
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
    fn check_packit_group(&self, config: &Config) -> Result<Option<Issue>> {
        // We don't need the packit group if multiuser mode is not enabled
        if config.multiuser && !does_packit_group_exist()? {
            return Ok(Some(Issue::MissingPackitGroup));
        }

        Ok(None)
    }

    /// Checks for directories which shouldn't be in the prefix/packages directory.
    /// This wil be any directory which is empty or doesn't have `<package-name>/<version>`.
    /// Returns None if no stray directories are found, `Issue::StrayDirectories` otherwise.
    fn check_stray_directories(&self, config: &Config) -> Result<Option<Issue>> {
        let package_directory = config.prefix_directory.join("packages");
        let mut strays = HashSet::new();
        for directory in fs::read_dir(package_directory)? {
            let directory = directory?;
            if !directory.path().is_dir() {
                strays.insert(directory.path());
                continue;
            }

            // Try to get the package name
            let package_name = directory.file_name();
            let Some(package_name) = package_name.to_str() else {
                strays.insert(directory.path());
                continue;
            };

            // Try to create a `PackageName`
            if PackageName::from_str(package_name).is_err() {
                strays.insert(directory.path());
                continue;
            }

            // Check if the name directory is empty
            if directory_is_empty(&directory.path())? {
                strays.insert(directory.path());
                continue;
            }

            for version_directory in fs::read_dir(directory.path())? {
                let version_directory = version_directory?;
                if !version_directory.path().is_dir() {
                    strays.insert(version_directory.path());
                    continue;
                }

                // Try to get the version name
                let version_name = version_directory.file_name();
                let Some(version_str) = version_name.to_str() else {
                    strays.insert(version_directory.path());
                    continue;
                };

                // Try to create a `Version`
                if Version::from_str(version_str).is_err() {
                    strays.insert(version_directory.path());
                    continue;
                };

                // Check if the version directory is empty
                if directory_is_empty(&version_directory.path())? {
                    strays.insert(version_directory.path());
                    continue;
                }
            }
        }

        if strays.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::StrayDirectories(strays)))
    }

    /// Gets the package version meta, or `None` if the provider cannot be found.
    fn get_package_version_meta(&self, package_id: &PackageId, package: &InstalledPackageVersion) -> Result<Option<PackageVersionMeta>> {
        let repository = Repository::new(&package.metadata_repository_url, &package.metadata_repository_provider);
        let Some(provider) = create_metadata_provider(&repository) else {
            return Ok(None);
        };

        Ok(Some(provider.read_package_version(&package_id.name, &package_id.version)?))
    }

    /// Get the issues found states.
    /// Returns true if issues are found, false otherwise.
    pub fn issues_found(&self) -> bool {
        self.issues_found
    }

    /// Reverses the initial checks counter by 1. Except if the current is 0.
    /// Returns the new value of current_intial_check.
    pub fn reverse_initial_check(&mut self) -> usize {
        if self.current_intial_check > 0 {
            self.current_intial_check -= 1;
        }

        self.current_intial_check
    }

    /// Reverses the checks counter by 1. Except if the current is 0.
    /// Returns the new value of `current_check`.
    pub fn reverse_check(&mut self) -> usize {
        if self.current_check > 0 {
            self.current_check -= 1;
        }

        self.current_check
    }

    /// Gets the current initial check index.
    pub fn get_initial_check_index(&self) -> usize {
        self.current_intial_check
    }

    /// Gets the current check index.
    pub fn get_check_index(&self) -> usize {
        self.current_check
    }
}
