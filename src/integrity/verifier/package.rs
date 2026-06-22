// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    cli::display::logging::{debug, warning},
    config::{Config, Repository},
    installer::{
        self,
        types::{Dependency, PackageId, PackageName},
    },
    integrity::{Issue, error::Result, utils::get_storage_packages},
    packager,
    platforms::Target,
    register::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    repositories::{
        provider::{self, create_metadata_provider},
        types::{Checksum, PackageVersionMeta},
    },
    utils::{io::directory_is_empty, ioerror::IOResultExt},
};

/// Checks for alterations in the given packages using a checksum which is compared to the checksum from the prebuild.
/// Returns an alteration issue or `None` if no packages can be found that are altered.
#[expect(unused_variables, unreachable_code)]
pub fn check_alterations(packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
    // TODO: For now skip this check, because it will never work (yet)
    return Ok(None);
    warning!("This is an experimental check, issues from this check could be inaccurate.");

    let mut altered = Vec::new();
    for package_id in packages {
        if check_package_alterations(package_id, register, config)? {
            altered.push(package_id.clone());
        }
    }

    if altered.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::AlteredPackage(altered)))
}

/// Checks for alterations in a single package using a checksum which is compared to the checksum from the prebuild.
/// Returns true if the package was altered, false if not.
#[expect(unused_variables, unreachable_code)]
fn check_package_alterations(package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<bool> {
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

/// Checks if the given packages in the register also exist in the package storage in the prefix directory.
/// Returns a storage consistency issue or None if there are no packages missing from storage.
pub fn check_storage_consistency(packages: &Vec<PackageId>, config: &Config) -> Result<Option<Issue>> {
    let mut missing = Vec::new();
    for package_id in packages {
        if !package_storage_is_consistent(&package_id, config)? {
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
fn package_storage_is_consistent(package_id: &PackageId, config: &Config) -> Result<bool> {
    let installed_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

    // Check if the directory exists, if so return true
    if fs::exists(&installed_directory).err_with_path("check existance of", &installed_directory)?
        && !directory_is_empty(&installed_directory)?
    {
        return Ok(true);
    }

    Ok(false)
}

/// Checks if the given packages in storage also exist in the register.
/// Note that this is a package related check, but not from the register and it would be weird to only check the specified packages.
/// As we would get all the packages from storage and then ignore unspecified packages (failing when no packages are specified by the user).
/// Returns a register consistency issue or None if there are packages missing from the register.
pub fn check_register_consistency(register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
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

/// Checks for the given packages if the package active is valid.
pub fn check_invalid_active(packages: &HashSet<PackageName>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
    let mut invalid_active = Vec::new();
    for package_name in packages {
        if let Some(package) = check_invalid_package_active(package_name, register, config)? {
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
fn check_invalid_package_active(package_name: &PackageName, register: &PackageRegister, config: &Config) -> Result<Option<PackageName>> {
    let Some(package) = register.get_package(package_name) else {
        return Ok(None);
    };

    let active_directory = config.prefix_directory.join("active").join(package_name);
    if !fs::exists(&active_directory).err_with_path("check existance of", &active_directory)? {
        return Ok(Some(package_name.clone()));
    }

    let active_link = fs::read_link(&active_directory).err_with_path("read link", active_directory)?;
    if active_link != config.prefix_directory.join("packages").join(package_name).join(package.active_version.to_string()) {
        return Ok(Some(package_name.clone()));
    }

    Ok(None)
}

/// Checks the given packages for a forbidden link. Where a forbidden link is a package which is symlinked
/// while it shouldn't be according to the repository metadata.
pub fn check_forbidden_link(packages: &Vec<PackageId>, register: &PackageRegister) -> Result<Option<Issue>> {
    let mut forbidden = Vec::new();
    for package_id in packages {
        if let Some(package) = check_forbidden_package_link(package_id, register)? {
            forbidden.push(package);
        }
    }

    if forbidden.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::ForbiddenLink(forbidden)))
}

/// Checks a given package for a forbidden link. Where a forbidden link is a package which is symlinked
/// while it shouldn't be according to the repository metadata.
fn check_forbidden_package_link(package_id: &PackageId, register: &PackageRegister) -> Result<Option<PackageName>> {
    let Some(package_version) = register.get_package_version(package_id) else {
        return Ok(None);
    };

    // Check if this package is allowed to be symlinked
    // Assume the package version meta can be found (otherwise no issue is returned)
    let link_allowed = match get_package_version_meta(package_id, package_version)? {
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
        return Ok(None);
    }

    let Some(package) = register.get_package(&package_id.name) else {
        return Ok(None);
    };

    if !package.symlinked {
        return Ok(None);
    }

    Ok(Some(package_id.name.clone()))
}

/// Checks for the given packages if symlinks are missing.
/// Returns an `Issue::MissingLinks` if symlinks are missing, None otherwise.
pub fn check_missing_link(packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
    let mut missing = Vec::new();
    for package_id in packages {
        if check_missing_package_link(package_id, register, config)? {
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
fn check_missing_package_link(package_id: &PackageId, register: &PackageRegister, config: &Config) -> Result<bool> {
    let Some(package) = register.get_package(&package_id.name) else {
        return Ok(false);
    };

    // Return early if the package is not the active version or if the packages shouldn't be symlinked
    if package.active_version != package_id.version || !package.symlinked {
        return Ok(false);
    }

    let package_path = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());
    for directory_name in installer::SYMLINK_DIRECTORIES {
        let symlink_directory = config.prefix_directory.join(directory_name);
        let directory = package_path.join(directory_name);

        // Continue if the directory doesn't exist in the package
        if !fs::exists(&directory).err_with_path("check existance of", &directory)? {
            continue;
        }

        if check_symlinks(&directory, &symlink_directory)? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Checks if a symlink can be found for the given directory.
/// Returns true if a symlink cannot be found, false otherwise.
fn check_symlinks(directory: &PathBuf, symlink_directory: &Path) -> Result<bool> {
    for file in fs::read_dir(directory).err_with_path("read", directory)? {
        let file = file.err_with_path("iterate", directory)?;
        let file_path = file.path();

        // Recurse
        if file_path.is_dir() {
            if check_symlinks(&file_path, &symlink_directory.join(file.file_name()))? {
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

/// Checks for the given packages if they have invalid dependents. Where an invalid dependent is a package which doesn't
/// exist or a package which doesn't have the given package as a dependency.
/// Returns a `Vec<(PackageId, PackageId)>`, with format `<child> : <dependent>`.
pub fn check_invalid_dependents(packages: &Vec<PackageId>, register: &PackageRegister) -> Option<Issue> {
    let mut invalid_dependents = Vec::new();
    for package_id in packages {
        invalid_dependents.extend(check_invalid_package_dependents(package_id, register));
    }

    if invalid_dependents.is_empty() {
        return None;
    }

    Some(Issue::InvalidDependents(invalid_dependents))
}

/// Checks if a given package has invalid dependents. Where an invalid dependent is a package which doesn't
/// exist or a package which doesn't have the given package as a dependency.
/// Returns a `Vec<(PackageId, PackageId)>`, with format `<child> : <dependent>`.
fn check_invalid_package_dependents(package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
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

/// Checks for the given packages if they miss any of their dependents.
pub fn check_missing_dependents(packages: &Vec<PackageId>, register: &PackageRegister) -> Option<Issue> {
    let mut missing_dependents = Vec::new();
    for package_id in packages {
        missing_dependents.extend(check_missing_package_dependents(package_id, register));
    }

    if missing_dependents.is_empty() {
        return None;
    }

    Some(Issue::MissingDependents(missing_dependents))
}

/// Check for the given package if its dependencies have it as a dependent.
fn check_missing_package_dependents(package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
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
pub fn check_missing_dependencies(packages: &Vec<PackageId>, register: &PackageRegister) -> Result<Option<Issue>> {
    let mut missing = Vec::new();
    for package_id in packages {
        let Some(package) = register.get_package_version(package_id) else {
            continue;
        };

        missing.extend(missing_dependencies_impl(package)?);
    }

    if missing.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::MissingDependencies(missing)))
}

/// Checks if a given package misses dependencies in the register according to the repository metadata.
/// Returns a list of missing dependencies for the given package (can be empty).
fn missing_dependencies_impl(package: &InstalledPackageVersion) -> Result<Vec<(PackageId, Dependency)>> {
    let package_id = &package.package_id;
    let mut missing = Vec::new();

    // Assume the package version meta can be found (otherwise no issue is returned)
    let Some(package_version_meta) = get_package_version_meta(package_id, package)? else {
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

/// Checks for invalid dependencies in the given packages.
/// Returns an `Issue::InvalidDependencies` with the invalid dependencies, or `None` if no dependencies are invalid.
pub fn check_invalid_dependencies(packages: &Vec<PackageId>, register: &PackageRegister) -> Result<Option<Issue>> {
    let mut invalid = Vec::new();
    for package_id in packages {
        let Some(package) = register.get_package_version(package_id) else {
            continue;
        };

        invalid.extend(invalid_dependencies_impl(package)?);
    }

    if invalid.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::InvalidDependencies(invalid)))
}

/// Checks if a given package has invalid dependencies in the register according to the repository metadata.
/// Returns a list of invalid dependencies for the given package (can be empty).
fn invalid_dependencies_impl(package: &InstalledPackageVersion) -> Result<Vec<(PackageId, PackageId)>> {
    let package_id = &package.package_id;
    let mut invalid = Vec::new();

    // Assume the package version meta can be found (otherwise no issue is returned)
    let Some(package_version_meta) = get_package_version_meta(package_id, package)? else {
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

/// Checks the completeness of the depedency trees from the given packages.
/// Returns a dependency tree issue or `None` if there are no packages missing from the dependency trees.
pub fn check_dependency_tree(packages: &Vec<PackageId>, register: &PackageRegister) -> Option<Issue> {
    let mut missing = Vec::new();
    for package_id in packages {
        missing.extend(check_package_dependency_tree(package_id, register));
    }

    if missing.is_empty() {
        return None;
    }

    Some(Issue::BrokenTree(missing))
}

/// Checks the completeness of the dependency tree from a specific package.
/// Returns a list of missing packages, can be empty if there are no packages missing from the tree.
fn check_package_dependency_tree(package_id: &PackageId, register: &PackageRegister) -> Vec<(PackageId, PackageId)> {
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
        missing.extend(check_package_dependency_tree(dependency, register));
    }

    missing
}

/// Gets the package version meta, or `None` if the provider cannot be found.
fn get_package_version_meta(package_id: &PackageId, package: &InstalledPackageVersion) -> Result<Option<PackageVersionMeta>> {
    let repository = Repository::new(&package.metadata_repository_url, &package.metadata_repository_provider);
    let Some(provider) = create_metadata_provider(&repository) else {
        return Ok(None);
    };

    Ok(Some(provider.read_package_version(&package_id.name, &package_id.version)?))
}
