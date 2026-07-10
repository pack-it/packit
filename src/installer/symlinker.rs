// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, iter, path::Path};

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::{
        error::{InstallerError, Result},
        types::{PackageId, PackageName},
    },
    platforms::symlink,
    register::package_register::PackageRegister,
    utils::{io, ioerror::IOResultExt},
};

/// All directories of a package that should be symlinked.
pub const SYMLINK_DIRECTORIES: &[&str] = &["bin", "include", "lib", "share", "gnubin"];

/// Manages symlink operations for installing, uninstalling and changing active and symlink states of packages.
pub struct Symlinker<'a> {
    config: &'a Config,
}

impl<'a> Symlinker<'a> {
    /// Creates a new `Symlinker`.
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Creates symlinks from Packit's "bin", "include", "lib", "share", etc. directories to
    /// the "bin", "include", "lib", "share", etc. directories in a given package directory.
    /// Could return an IO error.
    pub fn create_symlinks(&self, package_directory: &Path, overwrite: bool) -> Result<()> {
        let prefix_dir = Path::new(&self.config.prefix_directory);

        // Symlink package directories bin, include, lib, share, etc.
        for dir_name in SYMLINK_DIRECTORIES {
            let package_dir_path = package_directory.join(dir_name);
            let prefix_dir_path = prefix_dir.join(dir_name);

            io::create_folder_symlinks(&package_dir_path, &prefix_dir_path, overwrite)?;
        }

        Ok(())
    }

    /// Sets a package to active and creates the appropiate symlinks for it, based on the `should_symlink` parameter.
    /// Could return an `InstallerError::PackageNotFound`, a `RegisterError` or an IO error.
    pub fn set_active(&self, register: &mut PackageRegister, package_id: &PackageId, should_symlink: bool) -> Result<()> {
        // Get package to set to active
        let package_version = match register.get_package_version(package_id) {
            Some(package) => package,
            None => {
                warning!("Cannot get installed package from installed storage... Please check installation with 'pit list'");
                return Ok(());
            },
        };

        let global_active_path = Path::new(&self.config.prefix_directory).join("active");
        let active_path = global_active_path.join(&package_version.package_id.name);

        let package_directory = self.config.prefix_directory.join("packages").join(&package_id.name);

        // Remove symlinks for all package symlinked directories
        for dir_name in SYMLINK_DIRECTORIES.iter().chain(iter::once(&"active")) {
            let prefix_dir_path = self.config.prefix_directory.join(dir_name);

            io::remove_symlinks(&prefix_dir_path, &package_directory)?;
        }

        // Create active symlink
        fs::create_dir_all(&global_active_path).err_with_path("create dirs", &global_active_path)?;
        symlink::create_symlink(&package_version.install_path, &active_path)?;

        // Only create new symlinks if we should symlink
        if should_symlink {
            self.create_symlinks(Path::new(&package_version.install_path), false)?;
        }

        // Updates the active version and sets its symlinked state
        match register.get_package_mut(&package_id.name) {
            Some(package) => {
                package.active_version = package_id.version.clone();
                package.symlinked = should_symlink;
            },
            None => {
                return Err(InstallerError::PackageNotFound {
                    package_name: package_id.name.clone(),
                    version: None,
                });
            },
        }

        // Save package storage
        register.save_to(&PackageRegister::get_path(&self.config.prefix_directory))?;

        Ok(())
    }

    /// Unlinks a package based on a given package name.
    /// Could return an `InstallerError::PackageNotFound`, a `RegisterError` or an IO error.
    pub fn unlink_package(&self, register: &mut PackageRegister, package_name: &PackageName) -> Result<()> {
        let package = register.get_package(package_name).ok_or(InstallerError::PackageNotFound {
            package_name: package_name.clone(),
            version: None,
        })?;

        // Check if the package is already symlinked
        if !package.symlinked {
            return Ok(());
        }

        // Get active package version
        let package_version = package.get_package_version(&package.active_version).ok_or(InstallerError::PackageNotFound {
            package_name: package_name.clone(),
            version: Some(package.active_version.clone()),
        })?;

        // Remove all symlinks except for those in the active directory
        for entry in fs::read_dir(&self.config.prefix_directory).err_with_path("read", &self.config.prefix_directory)? {
            let entry = entry.err_with_path("iterate", &self.config.prefix_directory)?;

            let file_type = entry.file_type().err_with_path("get filetype of", &entry.path())?;
            if file_type.is_dir() && entry.file_name() != "active" {
                io::remove_symlinks(&entry.path(), &package_version.install_path)?;
            }
        }

        // Update symlinked state in package register
        match register.get_package_mut(package_name) {
            Some(package) => package.symlinked = false,
            None => {
                warning!("Cannot get installed package after changing symlinks, please try running pit fix to fix your installation");
                return Err(InstallerError::PackageNotFound {
                    package_name: package_name.clone(),
                    version: None,
                });
            },
        };

        // Save package register
        register.save_to(&PackageRegister::get_path(&self.config.prefix_directory))?;

        Ok(())
    }
}
