use crate::cli::display::{ask_user, DisplayLoad};
use crate::config::Config;
use crate::installed_packages::InstalledPackageStorage;
use crate::installer::error::Result;
use crate::installer::{error::InstallerError, unpack::unpack};
use crate::repositories::manager::RepositoryManager;
use crate::repositories::types::{Package, PackageVersion};
use crate::target_architecture::TARGET_ARCHITECTURE;
use crate::verifier::package_exists;

use std::fs;

/// The installer of Packit, managing the installation of packages on the system.
pub struct Installer<'a> {
    config: &'a Config,
}

impl<'a> Installer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Installs the given package and its dependencies.
    /// TODO: Maybe move some of the logic
    pub fn install(&self, manager: &RepositoryManager, package_name: &String, version: Option<String>) -> Result<()> {
        let (_, package) = manager.read_package(package_name)?;

        // Use the latest version if the version isn't specified
        let version = match version {
            Some(version) => version,
            None => package
                .latest_versions
                .get(TARGET_ARCHITECTURE)
                .expect("Temporary expect")
                .to_string(),
        };

        // TODO: Check if the version is already installed (currently it overwrites what already exists)

        // Get package version info for its target
        let (repository_id, package_version) = manager.read_package_version(&package_name, &version)?;
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

        // TODO: Should download include reading the response to bytes?
        display.show_finish("Downloading ".to_string() + package_name + " successful");

        // Install the package in the correct directory
        match target.installer_type.as_str() {
            "unpack" => unpack(bytes, &self.get_current_install_directory(package_name, &Some(version)))?,
            _ => {},
        }

        self.mark(&package, &package_version, &repository_id)?;
        Ok(())
    }

    fn mark(&self, package: &Package, package_version: &PackageVersion, repository_id: &String) -> Result<()> {
        // Mark package is installed
        // TODO: Adjust storage directory
        let mut installed_storage = InstalledPackageStorage::from(&(self.config.install_directory.to_string() + "/info.toml"))?;
        installed_storage.add_package(
            &package,
            &package_version,
            &self.config.repositories.get(repository_id).expect("Expected repository in config"),
            &self.config.install_directory,
        );
        installed_storage.save_to(&(self.config.install_directory.to_string() + "/info.toml"))?;
        Ok(())
    }

    /// TODO: Doesn't yet uninstall unused dependecies
    pub fn uninstall(&self, package_name: &String, version: Option<String>) -> Result<()> {
        let installed_storage = InstalledPackageStorage::from(&(self.config.install_directory.to_string() + "/info.toml"))?;
        let installed_versions = installed_storage.get_package_versions(package_name);

        // Make sure the package exists (first in info.toml and then in actual directory)
        // TODO: Should we assume the info.toml file is correct (otherwise why do we even have it)?
        // In case of none specified version check if there is at least one package with this package name
        // TODO: Make a method that syncs info.toml and actuall directory (in case of out of sync)
        if installed_versions.is_empty() || !package_exists(package_name, &version) {
            // Return error, package doesn't exist
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.clone(),
                version: version.unwrap_or("any".to_string()),
            });
        }

        // Aks user if he/she wants to continue when version isn't specified
        if !ask_user("Version is not specified, do you wish to uninstall all versions of this package?") {
            return Ok(());
        }

        // Remove all specified versions
        // Delete the version directories of a package
        match fs::remove_dir_all(self.get_current_install_directory(package_name, &version)) {
            Ok(_) => {}, // TODO: Log succes with display
            Err(e) => {
                return Err(InstallerError::UninstallError {
                    package_name: package_name.clone(),
                    e,
                })
            },
        };

        // Remove the package directory if it's empty (no versions left)
        let package_directory = self.get_current_install_directory(package_name, &Option::None);
        if fs::read_dir(&package_directory).unwrap().count() == 0 {
            fs::remove_dir(package_directory).map_err(|e| InstallerError::RemovalError(e))?;
        }

        // Delete the package info
        self.unmark(package_name, &version)?;

        Ok(())
    }

    fn unmark(&self, package_name: &String, version: &Option<String>) -> Result<()> {
        let info_directory = self.config.install_directory.to_string() + "/info.toml";
        let mut installed_storage = InstalledPackageStorage::from(&info_directory)?;
        installed_storage.remove_package(package_name, version);
        installed_storage.save_to(&info_directory)?;
        Ok(())
    }

    /// Gets the install directory for a specific package with version.
    /// If the version isn't specified it returns the package directory without version.
    fn get_current_install_directory(&self, package_name: &String, version: &Option<String>) -> String {
        if let Some(version) = version {
            return self.config.install_directory.to_string() + "/" + package_name + "/" + version;
        }
        self.config.install_directory.to_string() + "/" + package_name
    }
}
