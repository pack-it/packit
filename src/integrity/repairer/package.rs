// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, str::FromStr};

use crate::{
    cli::display::{logging::warning, styled::Styled},
    config::Config,
    installer::{
        Installer, InstallerOptions, Symlinker,
        types::{Dependency, OptionalPackageId, PackageId, PackageName, Version},
    },
    integrity::{
        error::{Result, VerifierError},
        utils::get_storage_packages,
    },
    packager,
    platforms::{
        Target,
        symlink::{self, SymlinkError},
    },
    register::package_register::PackageRegister,
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        types::{Checksum, PackageVersionMeta},
    },
    utils::ioerror::IOResultExt,
};

/// Fixes broken dependency trees by installing the missing packages.
pub fn fix_broken_tree(
    missing: Vec<(PackageId, PackageId)>,
    register: &mut PackageRegister,
    config: &Config,
    manager: &RepositoryManager,
) -> Result<()> {
    for (package, missing_package) in missing {
        let package_version = match register.get_package_version_mut(&missing_package) {
            Some(package_version) => package_version,
            None => {
                // Install the package
                let installer_options = InstallerOptions::default().skip_symlinking(true);
                let mut installer = Installer::new(config, register, manager, installer_options);
                installer.install(&missing_package.clone().into())?;

                // Assume installation worked
                let Some(package_version) = register.get_package_version_mut(&missing_package) else {
                    continue;
                };

                package_version
            },
        };

        // Fix the dependencies directory
        let dependencies_dir = config.prefix_directory.join("dependencies").join(package.to_string()).join(missing_package.name);
        match symlink::remove_symlink(&dependencies_dir) {
            Ok(_) => {},
            Err(SymlinkError::NonSymlink) => {},
            Err(e) => Err(e)?,
        }

        // Insert the package which misses the dependency as dependent
        package_version.dependents.insert(package);

        symlink::create_symlink(&package_version.install_path, &dependencies_dir)?;
    }

    Ok(())
}

/// Fixes inconsistent storage by temporarily removing the missing package from the register and then re-installing the packages.
pub fn fix_inconsistent_storage(
    missing: Vec<PackageId>,
    register: &mut PackageRegister,
    config: &Config,
    manager: &RepositoryManager,
) -> Result<()> {
    for missing_package in missing {
        // Gather the package settings before removing the package from the register
        let (symlinked, active) = match register.get_package(&missing_package.name) {
            Some(package) => (package.symlinked, package.active_version == missing_package.version),
            None => {
                warning!("Inconsistent package cannot be found in Register.toml anymore, eventhough it was found before.");
                (false, false)
            },
        };

        // Temporarily remove the package from the register
        register.remove_package_version(&missing_package);

        let installer_options = InstallerOptions::default().skip_symlinking(!symlinked).skip_active(!active);
        let mut installer = Installer::new(config, register, manager, installer_options);

        installer.install(&missing_package.into())?;
    }

    Ok(())
}

/// Fixes an inconsistent register by gathering still existing data from the Packit directories.
pub fn fix_inconsistent_register(
    missing: HashSet<PackageId>,
    register: &mut PackageRegister,
    config: &Config,
    manager: &RepositoryManager,
) -> Result<()> {
    let storage_packages = get_storage_packages(config)?;
    let active_directory = config.prefix_directory.join("active");
    let bin_directory = config.prefix_directory.join("bin");
    let package_directory = config.prefix_directory.join("packages");
    for package_id in &missing {
        // Skip if the package already exists in the register (from recursive step)
        if register.get_package_version(package_id).is_some() {
            continue;
        }

        // Use checksum to check if a prebuild was used (use checksum to make sure it's the same prebuild)
        let install_path = &package_directory.join(&package_id.name).join(package_id.version.to_string());
        let (repository_id, _, package_version_meta) = manager.read_package_and_version(&package_id.clone().into(), &Target::current())?;
        let revisions = package_version_meta.get_revision_count();
        let prebuild_id = Target::current().architecture.to_string();
        let used_prebuild = match manager.get_prebuild_checksum(&repository_id, package_id, revisions, &prebuild_id) {
            Ok(Some(correct_checksum)) => {
                let compressed = packager::compress(install_path)?;
                let checksum = Checksum::from_bytes(&compressed);
                correct_checksum == checksum
            },
            Ok(None) => false,
            Err(RepositoryError::RepositoryNotFoundError { .. }) => false,
            Err(e) => Err(e)?,
        };

        // Figure out the active version
        let active_link_path = active_directory.join(&package_id.name);
        let active_target = fs::read_link(&active_link_path).err_with_path("read link", active_link_path)?;
        let target_name = active_target.file_name().ok_or(VerifierError::InvalidSymlink)?;
        let version = Version::from_str(target_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;

        // Check if the package is the active package version
        let active = package_id.version == version;

        // Figure out if symlinked
        let symlinked = fs::symlink_metadata(bin_directory.join(&package_id.name)).is_ok();

        // Get information with the manager
        // Note that this information is valid, but necessarily the same as before the issue arised
        let package_meta = manager.read_repo_package(&repository_id, &package_id.name)?;
        let dependencies = get_latest_satisfying_packages(&package_version_meta, &storage_packages);
        let source_repository = config.repositories.get(&repository_id).expect("Expected repository in config");

        // Make sure that all dependencies are registered as well
        let missing_dependencies = dependencies.iter().filter(|d| register.get_package_version(d).is_none()).cloned().collect();
        fix_inconsistent_register(missing_dependencies, register, config, manager)?;

        register.add_package(
            &package_meta,
            &package_version_meta,
            dependencies,
            source_repository,
            install_path,
            symlinked,
            active,
            used_prebuild,
        )?;
    }

    Ok(())
}

/// Gets the latest satisfying dependencies for a given package from the given storage packages.
/// Assumes that all dependencies are present in the given storage packages.
/// Returns a `HashSet` with all the latest packages which satisfy the dependencies of the given package id.
fn get_latest_satisfying_packages(package_version_meta: &PackageVersionMeta, storage_packages: &HashSet<PackageId>) -> HashSet<PackageId> {
    // Find the latest satisfying dependency
    let mut dependencies = HashSet::new();
    for dependency in &package_version_meta.dependencies {
        let mut latest: Option<&PackageId> = None;
        for package in storage_packages {
            if !dependency.satisfied(&package.name, &package.version) {
                continue;
            }

            match latest {
                Some(id) if id.version < package.version => latest = Some(package),
                Some(_) => {},
                None => latest = Some(package),
            }
        }

        // This assumes that all dependencies can be satisfied with packages in the given storage packages
        if let Some(latest) = latest {
            dependencies.insert(latest.clone());
        }
    }

    dependencies
}

/// Fixes missing dependencies by adding them to the register.
/// If an installed satisfying package can be found it's used, otherwise the latest version is used instead.
pub fn fix_missing_dependencies(
    missing: Vec<(PackageId, Dependency)>,
    register: &mut PackageRegister,
    manager: &RepositoryManager,
) -> Result<()> {
    for (package_id, missing_dependency) in missing {
        // Try to find an installed satisfying package, if not found use latest version
        let package = match register.get_latest_satisfying_package(&missing_dependency) {
            Some(package) => package.package_id.clone(),
            None => {
                let (repository_id, package_metadata) = manager.read_package(missing_dependency.get_name())?;
                let version_metadata = manager.read_latest_supported_version(&repository_id, &package_metadata, &Target::current())?;
                PackageId::new(missing_dependency.get_name().clone(), version_metadata.version)
            },
        };

        // Set the dependency in the register
        let Some(package_version) = register.get_package_version_mut(&package_id) else {
            continue;
        };

        package_version.dependencies.insert(package);
    }

    Ok(())
}

/// Fixes the invalid dependencies issue.
pub fn fix_invalid_dependencies(invalid: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) -> Result<()> {
    for (package, invalid_dependency) in invalid {
        // Assume the given package exists (continue otherwise)
        let Some(package_version) = register.get_package_version_mut(&package) else {
            continue;
        };

        package_version.dependencies.remove(&invalid_dependency);
    }

    Ok(())
}

/// Fixes the missing dependents issue.
pub fn fix_missing_dependents(missing: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) {
    for (child, parent) in missing {
        let Some(package_version) = register.get_package_version_mut(&child) else {
            warning!("Could not fix missing dependents for {}", child.style());
            continue;
        };

        package_version.dependents.insert(parent);
    }
}

/// Fixes the invalid dependents issue.
pub fn fix_invalid_dependents(invalid: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) {
    for (child, parent) in invalid {
        let Some(package_version) = register.get_package_version_mut(&child) else {
            warning!("Could not fix invalid dependents for {}", child.style());
            continue;
        };

        package_version.dependents.remove(&parent);
    }
}

/// Fixes the invalid active issue.
pub fn fix_invalid_active(invalid: Vec<PackageName>, register: &mut PackageRegister, config: &Config) -> Result<()> {
    let symlinker = Symlinker::new(config);
    for package_name in invalid {
        let Some(package) = register.get_package_mut(&package_name) else {
            warning!("Could not fix invalid active for {}", package_name.style());
            continue;
        };

        // Set the active to the latest installed version of the package
        if let Some(version) = package.versions.keys().max() {
            package.active_version = version.clone();
        }

        let package_id = PackageId::new(package_name.clone(), package.active_version.clone());
        let symlinked = package.symlinked;
        symlinker.set_active(register, &package_id, symlinked)?;
    }

    Ok(())
}

/// Fixes the forbidden link issue.
pub fn fix_forbidden_link(forbidden: Vec<PackageName>, register: &mut PackageRegister, config: &Config) -> Result<()> {
    let symlinker = Symlinker::new(config);

    // Unlink all packages which shouldn't be symlinked
    for package_name in forbidden {
        symlinker.unlink_package(register, &package_name)?;
    }

    Ok(())
}

/// Fixes the missing links issue.
pub fn fix_missing_links(missing: Vec<PackageName>, register: &mut PackageRegister, config: &Config) -> Result<()> {
    let symlinker = Symlinker::new(config);

    // Re-link all packages which have missing symlinks
    for package_name in &missing {
        let Some(package) = register.get_package(package_name) else {
            warning!("Could not find package {} for fix, skipping", package_name.style());
            continue;
        };

        let package_id = PackageId::new(package_name.clone(), package.active_version.clone());
        let Some(package_version) = register.get_package_version(&package_id) else {
            warning!("Could not find package {} for fix, skipping", package_id.style());
            continue;
        };

        let install_path = package_version.install_path.clone();
        symlinker.unlink_package(register, package_name)?;
        symlinker.create_symlinks(&install_path, false)?;

        if let Some(package) = register.get_package_mut(package_name) {
            package.symlinked = true;
        };
    }

    Ok(())
}

/// Tries to fix packages by re-installing those packages.
pub fn try_reinstall(packages: Vec<PackageId>, register: &mut PackageRegister, manager: &RepositoryManager, config: &Config) -> Result<()> {
    for package in packages {
        reinstall_package(&package, register, manager, config)?;
    }

    Ok(())
}

/// Uninstalls a package and then reinstalls it.
fn reinstall_package(package_id: &PackageId, register: &mut PackageRegister, manager: &RepositoryManager, config: &Config) -> Result<()> {
    // Figure out the active version
    let Some(package) = register.get_package(&package_id.name) else {
        return Ok(());
    };

    // Check if the package should be the active and symlinked package when installed
    let active = package_id.version == package.active_version;
    let symlinked = package.symlinked;

    // Gather dependents
    let Some(package_version) = register.get_package_version(package_id) else {
        return Ok(());
    };
    let dependents = package_version.dependents.clone();

    // Temporarily remove the package
    let installer_options = InstallerOptions::default().skip_symlinking(!package.symlinked).skip_active(true);
    let mut installer = Installer::new(config, register, manager, installer_options);
    installer.uninstall(&OptionalPackageId::from(package_id.clone()))?;

    // Re-install the package
    installer.install(&OptionalPackageId::from(package_id.clone()))?;

    // Re-add package as dependent
    for dependent in &dependents {
        let Some(package_version) = register.get_package_version_mut(dependent) else {
            return Ok(());
        };

        package_version.dependencies.insert(package_id.clone());
    }

    // Add the dependents to the package
    if let Some(package_version) = register.get_package_version_mut(package_id) {
        package_version.dependents = dependents;
    }

    // Set the package as the active package if active before
    if active {
        Symlinker::new(config).set_active(register, package_id, symlinked)?;
    }

    Ok(())
}
