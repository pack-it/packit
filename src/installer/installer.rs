use crate::{
    cli::display::{ask_user, DisplayLoad},
    config::Config,
    installed_packages::{InstalledPackage, InstalledPackageStorage},
    installer::{
        error::{InstallerError, Result},
        scripts::{self, ScriptError, SCRIPT_EXTENSION},
        unpack::unpack,
    },
    repositories::manager::RepositoryManager,
    target_architecture::TARGET_ARCHITECTURE,
};

use std::fs;

const TEMP_DIRECTORY: &str = "./temp";

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
            None => package.latest_versions.get(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?.to_string(),
        };

        // Check if this package version is already installed
        // TODO: Adjust info storage directory (also used in other places)
        if self.installed_storage.get_package(package_name, &version).is_some() {
            println!("Package {package_name} is already installed!");
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

        // Get bytes from response
        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => return Err(InstallerError::RequestError(e)),
        };

        display.show_finish("Downloading ".to_string() + package_name + " successful");

        let path_suffix = format!("{package_name}/{version}");

        // Unpack the package to the temp directory
        let unpack_directory = format!("{}/{path_suffix}", TEMP_DIRECTORY);
        unpack(bytes, &unpack_directory)?;

        let install_directory = format!("{}/{path_suffix}", self.config.install_directory);

        // Download and run pre install script if it exists
        let script_name = &target.preinstall_script;
        if let Some(script_path) = self.download_script("preinstall", script_name, package_name, &version, &repository_id, manager)? {
            scripts::run_pre_script(&script_path, &unpack_directory, self.config, &install_directory)?;
        }

        // Download and run build script
        let build_script_path = self
            .download_script("build", &target.build_script, package_name, &version, &repository_id, manager)?
            .ok_or(ScriptError::ScriptNotFound("build".into()))?;
        scripts::run_build_script(&build_script_path, &unpack_directory, self.config, &install_directory)?;

        // Add and save package to info storage
        let source_repository = self.config.repositories.get(&repository_id).expect("Expected repository in config");
        self.installed_storage.add_package(&package, &package_version, source_repository, &install_directory);

        // Download and run post install script if it exists
        let script_name = &target.postinstall_script;
        if let Some(script_path) = self.download_script("postinstall", script_name, package_name, &version, &repository_id, manager)? {
            scripts::run_post_script(&script_path, &install_directory, self.config)?;
        }

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

    fn download_script(
        &self,
        script_name: &str,
        target_script_name: &Option<String>,
        package_name: &str,
        version: &str,
        repository_id: &str,
        manager: &RepositoryManager,
    ) -> Result<Option<String>> {
        // Get name of the script
        let script_name = match target_script_name {
            Some(script) => script,
            None => &format!("{script_name}.{SCRIPT_EXTENSION}"),
        };

        // Download script
        let script_destination = format!("{}/{package_name}_{version}_{script_name}.{SCRIPT_EXTENSION}", TEMP_DIRECTORY);
        match manager.read_script(&repository_id, &package_name, &version, &script_name)? {
            Some(script_text) => scripts::save_script(&script_text, &script_destination)?,
            None => return Ok(None), // Script not found, so return None
        }

        // Script succesfully downloaded, so return script location
        Ok(Some(script_destination))
    }
}
