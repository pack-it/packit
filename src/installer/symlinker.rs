use std::{fs, path::Path};

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::{
        error::{InstallerError, Result},
        types::{PackageId, PackageName},
    },
    platforms::symlink,
    storage::package_register::PackageRegister,
    utils::io::{create_folder_symlinks, remove_symlinks},
};

/// Does symlink opperations for the install process.
pub struct Symlinker<'a> {
    config: &'a Config,
}

impl<'a> Symlinker<'a> {
    /// Creates a new `Symlinker`.
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Creates symlinks from Packit's "bin", "include", "lib" and "share" folders to
    /// the "bin", "include", "lib" and "share" folders in a given package directory.
    /// Could return an IO error.
    pub fn create_symlinks(&self, package_directory: &Path) -> Result<()> {
        let prefix_dir = Path::new(&self.config.prefix_directory);

        // Symlink directories bin, include, lib and share
        for dir_name in vec!["bin", "include", "lib", "share"] {
            let package_dir_path = package_directory.join(dir_name);
            let prefix_dir_path = prefix_dir.join(dir_name);

            create_folder_symlinks(&package_dir_path, &prefix_dir_path)?;
        }

        Ok(())
    }

    /// A wrapper around the `remove_symlinks` method from IO utils, which removes a symlinks in a given
    /// directory which have a given destination directory.
    /// Could return an IO error.
    pub(super) fn remove_symlinks(&self, search_dir: &Path, destination_dir: &Path) -> Result<()> {
        Ok(remove_symlinks(search_dir, destination_dir)?)
    }

    /// Sets a package to active and creates the appropiate symlinks for it, based on the `should_symlink` parameter.
    /// Could return a `InstallerError::PackageNotFound`, a `RegisterError` or an IO error.
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

        let package_install_path = package_version.install_path.clone();

        // Remove old symlinks
        let package_directory = self.config.prefix_directory.join("packages").join(&package_id.name);
        remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&package_directory))?;

        // Create active symlink
        fs::create_dir_all(global_active_path)?;
        symlink::create_symlink(&package_install_path, &active_path)?;

        // Only create new symlinks if we should symlink
        if should_symlink {
            self.create_symlinks(Path::new(&package_install_path))?;
        }

        // Updates the active version and sets its symlinked state
        match register.get_package_mut(&package_id.name) {
            Some(package) => {
                package.active_version = package_id.version.clone();
                package.symlinked = should_symlink;
            },
            None => {
                return Err(InstallerError::PackageNotFound {
                    package_name: package_id.name.to_string(),
                    version: None,
                });
            },
        }

        // Save package storage
        register.save_to(&PackageRegister::get_default_path(self.config))?;

        Ok(())
    }

    /// Unlinks a package based on a given package name.
    /// Could return a `InstallerError::PackageNotFound`, a `RegisterError` or an IO error.
    pub fn unlink_package(&self, register: &mut PackageRegister, package_name: &PackageName) -> Result<()> {
        let package = register.get_package(&package_name).ok_or(InstallerError::PackageNotFound {
            package_name: package_name.to_string(),
            version: None,
        })?;

        // Check if the package is already symlinked
        if !package.symlinked {
            return Ok(());
        }

        // Get active package version
        let package_version = package.get_package_version(&package.active_version).ok_or(InstallerError::PackageNotFound {
            package_name: package_name.to_string(),
            version: Some(package.active_version.to_string()),
        })?;

        let install_path = package_version.install_path.clone();

        // Remove all symlinks except for those in the active directory
        for entry in fs::read_dir(&self.config.prefix_directory)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() && entry.file_name() != "active" {
                remove_symlinks(&entry.path(), &install_path)?;
            }
        }

        // Update symlinked state in package register
        match register.get_package_mut(package_name) {
            Some(package) => package.symlinked = false,
            None => {
                warning!("Cannot get installed package after changing symlinks, please try running pit fix to fix your installation");
                return Err(InstallerError::PackageNotFound {
                    package_name: package_name.to_string(),
                    version: None,
                });
            },
        };

        // Save package register
        register.save_to(&PackageRegister::get_default_path(self.config))?;

        Ok(())
    }
}
