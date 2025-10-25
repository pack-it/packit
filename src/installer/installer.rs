use crate::cli::display::{ask_user, DisplayLoad};
use crate::config::Config;
use crate::installed_packages::{InstalledPackage, InstalledPackageStorage};
use crate::installer::error::Result;
use crate::installer::{error::InstallerError, unpack::unpack};
use crate::repositories::manager::RepositoryManager;
use crate::repositories::types::{Package, PackageVersion};
use crate::target_architecture::TARGET_ARCHITECTURE;

use std::fs;

// This is the directory relative to the install directory
const INFO_DIRECTORY: &str = "/info.toml";

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

        // TODO: Check if the version is already installed (currently it overwrites what already exists) (and for dependencies it adds them dubble)

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
        let directory = self.config.install_directory.to_string() + "/" + package_name + "/" + &version;
        match target.installer_type.as_str() {
            "unpack" => unpack(bytes, &directory)?,
            _ => {},
        }

        self.mark(&package, &package_version, &repository_id)?;
        Ok(())
    }

    fn mark(&self, package: &Package, package_version: &PackageVersion, repository_id: &String) -> Result<()> {
        // Mark package is installed
        // TODO: Adjust storage directory
        let mut installed_storage = InstalledPackageStorage::from(&(self.config.install_directory.to_string() + INFO_DIRECTORY))?;
        installed_storage.add_package(
            &package,
            &package_version,
            &self.config.repositories.get(repository_id).expect("Expected repository in config"),
            &self.config.install_directory,
        );
        installed_storage.save_to(&(self.config.install_directory.to_string() + INFO_DIRECTORY))?;
        Ok(())
    }

    /// TODO: Doesn't yet uninstall unused dependecies
    /// TODO: Give warning if user tries to delete a dependency
    /// TODO: Make a method that syncs info.toml and actuall directory (in case of out of sync)
    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&self, package_name: &String, version: Option<String>) -> Result<()> {
        let info_directory = self.config.install_directory.to_string() + INFO_DIRECTORY;
        let mut installed_storage = InstalledPackageStorage::from(&info_directory)?;
        let installed_versions = installed_storage.get_package_versions(package_name);

        // This determines the directory to remove. If there are multiple versions and the version is
        // specified only the specified version directory will be deleted. The entire package directory
        // is deleted if the version isn't specified or if the package directory only contains one version.
        match version {
            Some(version) => {
                self.uninstall_single(package_name, &version, &installed_versions)?;

                // Delete the package info
                installed_storage.remove_package_version(package_name, &version);
            },
            None => {
                // Ask the user if he/she wants to continue when version isn't specified and there are multiple versions installed
                let question = "Version is not specified, do you wish to uninstall all versions of this package?";
                if installed_versions.len() > 1 || !ask_user(question) {
                    return Ok(()); // TODO: Log skipped with display
                }

                self.uninstall_all(package_name, &installed_versions)?;

                // Delete the package info
                installed_storage.remove_package(package_name);
            },
        };

        // Save the new installed storage
        installed_storage.save_to(&info_directory)?;

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
