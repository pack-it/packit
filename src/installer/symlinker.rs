use std::{fs, path::Path};

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::{
        error::{InstallerError, Result},
        types::PackageId,
    },
    platforms::symlink,
    storage::package_register::PackageRegister,
};

pub struct Symlinker<'a> {
    config: &'a Config,
}

impl<'a> Symlinker<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn create_symlinks(&self, package_directory: &Path) -> Result<()> {
        let prefix_dir = Path::new(&self.config.prefix_directory);

        // Symlink directories bin, include, lib and share
        for dir_name in vec!["bin", "include", "lib", "share"] {
            let package_dir_path = package_directory.join(dir_name);
            let prefix_dir_path = prefix_dir.join(dir_name);

            self.create_folder_symlinks(&package_dir_path, &prefix_dir_path, true)?;
        }

        Ok(())
    }

    fn create_folder_symlinks(&self, source_dir: &Path, destination_dir: &Path, keep_subdirectories: bool) -> Result<()> {
        // Create destination if it does not exist
        if !destination_dir.exists() {
            fs::create_dir_all(&destination_dir)?;
        }

        // Skip symlinking if source does not exist
        if !source_dir.exists() {
            return Ok(());
        }

        // Symlink files
        for file in fs::read_dir(source_dir)? {
            let file = file?;

            let destination = destination_dir.join(file.file_name());

            // Handle directories
            if file.file_type()?.is_dir() {
                // If we want to keep subdirectories, create the symlinks for the subdirectory
                // TODO: Handle subdirectories properly
                if keep_subdirectories {
                    self.create_folder_symlinks(&file.path(), &destination, true)?;
                } else {
                    dbg!("Skipping subdirectory", file);
                }

                continue;
            }

            // Check if file already exists
            if fs::exists(&destination)? {
                warning!("Symlink {:?} already exists in {:?}", file.file_name(), destination_dir);
                continue;
            }

            // Symlink file in destination directory
            symlink::create_symlink(&file.path(), &destination)?;
        }

        Ok(())
    }

    /// Searches for symlinks with a certain destination (destinations inside of the destination are also a match).
    pub fn remove_symlinks(&self, search_dir: &Path, destination_dir: &Path) -> Result<()> {
        for file in fs::read_dir(search_dir)? {
            let file = file?;
            let file_type = file.file_type()?;

            if file_type.is_dir() {
                self.remove_symlinks(&file.path(), destination_dir)?;

                // Remove the directory if it is empty after removing symlinks
                if fs::read_dir(file.path())?.next().is_none() {
                    fs::remove_dir(file.path())?;
                }
            }

            if file_type.is_symlink() && fs::read_link(file.path())?.starts_with(destination_dir) {
                symlink::remove_symlink(&file.path())?
            }
        }

        Ok(())
    }

    /// Sets a package to active and create the appropiate symlinks for it
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
        self.remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&package_directory))?;

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
                    package_name: package_id.name.clone(),
                    version: None,
                })
            },
        }

        // Save package storage
        register.save_to(&PackageRegister::get_default_path())?;

        Ok(())
    }

    pub fn unlink_package(&self, register: &mut PackageRegister, package_name: &str) -> Result<()> {
        let package = register.get_package(&package_name).ok_or(InstallerError::PackageNotFound {
            package_name: package_name.into(),
            version: None,
        })?;

        // Check if the package is already symlinked
        if !package.symlinked {
            return Ok(());
        }

        // Get active package version
        let package_version = package.get_package_version(&package.active_version).ok_or(InstallerError::PackageNotFound {
            package_name: package_name.into(),
            version: Some(package.active_version.to_string()),
        })?;

        let install_path = package_version.install_path.clone();

        // Remove all symlinks except for those in the active directory
        for entry in fs::read_dir(&self.config.prefix_directory)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() && entry.file_name() != "active" {
                self.remove_symlinks(&entry.path(), &install_path)?;
            }
        }

        // Update symlinked state in package register
        match register.get_package_mut(package_name) {
            Some(package) => package.symlinked = false,
            None => {
                warning!("Cannot get installed package after changing symlinks, please try running pit fix to fix your installation");
                return Err(InstallerError::PackageNotFound {
                    package_name: package_name.into(),
                    version: None,
                });
            },
        };

        // Save package register
        register.save_to(&PackageRegister::get_default_path())?;

        Ok(())
    }
}
