use crate::cli::display::{ask_user, DisplayLoad};
use crate::config::Config;
use crate::installed_packages::{InstalledPackage, InstalledPackageStorage};
use crate::installer::error::Result;
use crate::installer::{error::InstallerError, unpack::unpack};
use crate::repositories::manager::RepositoryManager;
use crate::target_architecture::TARGET_ARCHITECTURE;

use std::fs;

/// The installer of Packit, managing the installation of packages on the system.
pub struct Installer<'a> {
    config: &'a Config,
    installed_storage: &'a mut InstalledPackageStorage,
}

impl<'a> Installer<'a> {
    /// Creates new installer
    pub fn new(config: &'a Config, installed_storage: &'a mut InstalledPackageStorage) -> Self {
        Self { config, installed_storage }
    }

    /// Installs the given package and its dependencies.
    pub fn install(&mut self, manager: &RepositoryManager, package_name: &String, version: Option<String>) -> Result<()> {
        let (repository_id, package) = manager.read_package(package_name)?;

        // Use the latest version if the version isn't specified
        let version = match version {
            Some(version) => version,
            None => package
                .latest_versions
                .get(TARGET_ARCHITECTURE)
                .ok_or(InstallerError::TargetError)?
                .to_string(),
        };

        // Check if this package version is already installed
        // TODO: Adjust info storage directory (also used in other places)
        if self.installed_storage.get_package(package_name, &version).is_some() {
            // TODO: Log already installed error with display. if it is a dependency, just log info, already installed, so skipping
            return Ok(());
        }

        // Get package version info for its target
        let package_version = manager.read_repo_package_version(&repository_id, &package_name, &version)?;
        let target = match package_version.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => return Err(InstallerError::TargetError),
        };

        // Install global package dependencies and platform specific packages (if there are any, can be empty)
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            self.install(manager, dependency, Option::None)?
        }

        // Show download
        let display = DisplayLoad::new();
        display.show("Downloading ".to_string() + package_name);

        // Request the data of the package
        let response = match reqwest::blocking::get(&target.url) {
            Ok(response) => response,
            Err(e) => {
                return Err(InstallerError::RequestError(e));
            },
        };

        //  Get bytes from response
        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => return Err(InstallerError::RequestError(e)),
        };

        display.show_finish("Downloading ".to_string() + package_name + " successful");

        // Install the package in the correct directory
        let directory = self.config.install_directory.to_string() + "/" + package_name + "/" + &version;
        match target.installer_type.as_str() {
            "unpack" => unpack(bytes, &directory)?,
            _ => {},
        }

        // Add and save package to info storage
        let source_repository = self.config.repositories.get(&repository_id).expect("Expected repository in config");
        self.installed_storage
            .add_package(&package, &package_version, source_repository, &self.config.install_directory);

        Ok(())
    }

    /// TODO: Doesn't yet uninstall unused dependecies (check dependencies of other packages (loop))
    /// TODO: Give warning if user tries to delete a dependency (check dependencies of other packages (loop))
    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&mut self, package_name: &String, version: Option<String>) -> Result<()> {
        let installed_versions = self.installed_storage.get_package_versions(package_name);

        // This determines the directory to remove. If there are multiple versions and the version is
        // specified only the specified version directory will be deleted. The entire package directory
        // is deleted if the version isn't specified or if the package directory only contains one version.
        match version {
            Some(version) => {
                self.uninstall_single(package_name, &version, &installed_versions)?;

                // Delete the package info
                self.installed_storage.remove_package_version(package_name, &version);
            },
            None => {
                // Ask the user if he/she wants to continue when version isn't specified and there are multiple versions installed
                let question = "Version is not specified, do you wish to uninstall all versions of this package?";
                if installed_versions.len() > 1 && !ask_user(question, true)? {
                    println!("Canceled uninstall of package: {package_name}");
                    return Ok(());
                }

                self.uninstall_all(package_name, &installed_versions)?;

                // Delete the package info
                self.installed_storage.remove_package(package_name);
            },
        };

        Ok(())
    }

    /// Checks if the directory exists. If so, it gets the remove directory for a package version, if there only exists one
    /// version it will return the package directory.
    fn uninstall_single(&self, package_name: &String, version: &String, installed_versions: &Vec<&InstalledPackage>) -> Result<()> {
        // Check if the specified package version exists.
        if !installed_versions.iter().any(|package| package.version == *version) {
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.clone(),
                version: version.clone(),
            });
        }

        // Remove entire package directory if there is only one version
        let directory: String;
        if installed_versions.len() == 1 {
            directory = self.config.install_directory.to_string() + "/" + package_name;
        } else {
            // The remove directory of a specific package version
            directory = self.config.install_directory.to_string() + "/" + package_name + "/" + version;
        }

        // Delete the determined directory
        self.remove_dir_all(&directory, package_name)?;

        Ok(())
    }

    // Checks if there exists at least one version of the specified package. If so, it returns the package directory.
    fn uninstall_all(&self, package_name: &String, installed_versions: &Vec<&InstalledPackage>) -> Result<()> {
        // Make sure at least on version exists
        if installed_versions.is_empty() {
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.clone(),
                version: "any".to_string(),
            });
        }

        // Delete the determined directory
        let directory = self.config.install_directory.to_string() + "/" + package_name;
        self.remove_dir_all(&directory, package_name)?;

        Ok(())
    }

    /// Wraps around the fs::remove_dir_all to map its error.
    fn remove_dir_all(&self, directory: &String, package_name: &String) -> Result<()> {
        match fs::remove_dir_all(directory) {
            Ok(_) => Ok(()), // TODO: Log succes with display
            Err(e) => Err(InstallerError::UninstallError {
                package_name: package_name.clone(),
                e,
            }),
        }
    }
}
