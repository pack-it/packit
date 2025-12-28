use crate::{
    cli::{ask_user, QuestionResponse, Spinner},
    config::Config,
    installed_packages::InstalledPackageStorage,
    installer::{
        error::{InstallerError, Result},
        scripts::{self, ScriptError, SCRIPT_EXTENSION},
        unpack::unpack,
    },
    repositories::manager::RepositoryManager,
    target_architecture::TARGET_ARCHITECTURE,
};

use std::fs;

/// The installer of Packit, managing the installation of packages on the system.
pub struct Installer<'a> {
    config: &'a Config,
    installed_storage: &'a mut InstalledPackageStorage,
    repository_manager: &'a RepositoryManager<'a>,
}

impl<'a> Installer<'a> {
    /// Creates new installer
    pub fn new(config: &'a Config, installed_storage: &'a mut InstalledPackageStorage, repository_manager: &'a RepositoryManager) -> Self {
        Self {
            config,
            installed_storage,
            repository_manager,
        }
    }

    /// Installs the given package and its dependencies.
    pub fn install(&mut self, package_name: &str, version: Option<String>) -> Result<()> {
        let (repository_id, package) = self.repository_manager.read_package(package_name)?;

        // Use the latest version if the version isn't specified
        let version = match version {
            Some(version) => version,
            None => package.latest_versions.get(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?.to_string(),
        };

        // Check if this package version is already installed
        if self.installed_storage.get_package(package_name, &version).is_some() {
            println!("Dependency '{} {}' already satisfied, continuing", package_name, version);
            return Ok(());
        }

        // Get package version info for its target
        let package_version = self.repository_manager.read_repo_package_version(&repository_id, &package_name, &version)?;
        let target = match package_version.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => return Err(InstallerError::TargetError),
        };

        // Install global package dependencies and platform specific packages (if there are any, can be empty)
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            self.install(dependency, None)?;
        }

        // Install global package build dependencies and platform specific build dependencies
        let build_dependencies = package_version.build_dependencies.iter().chain(target.build_dependencies.iter());
        for build_dependency in build_dependencies {
            // TODO: Delete build dependencies later, somewhere
            self.install(build_dependency, None)?;
        }

        // Show download
        let spinner = Spinner::new();
        spinner.show("Downloading ".to_string() + package_name);

        // Request the data of the package and get bytes
        let response = reqwest::blocking::get(&target.url)?;
        let bytes = response.bytes()?;

        spinner.finish("Downloading ".to_string() + package_name + " successful");

        let path_suffix = format!("{package_name}/{version}");

        // Unpack the package to the temp directory
        let unpack_directory = format!("{}/{path_suffix}", self.config.temp_directory);
        unpack(bytes, &unpack_directory)?;

        let install_directory = format!("{}/{path_suffix}", self.config.install_directory);

        // Download and run pre install script if it exists
        let script_name = package_version.get_preinstall_script_name(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;
        if let Some(script_path) = self.download_script("preinstall", &script_name, package_name, &version, &repository_id)? {
            scripts::run_pre_script(&script_path, &unpack_directory, self.config, &install_directory)?;
        }

        // Download and run build script
        let script_name = package_version.get_build_script_name(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;
        let build_script_path = self
            .download_script("build", &script_name, package_name, &version, &repository_id)?
            .ok_or(ScriptError::ScriptNotFound("build".into()))?;
        scripts::run_build_script(&build_script_path, &unpack_directory, self.config, &install_directory)?;

        // Add and save package to installed storage toml
        let source_repository = self.config.repositories.get(&repository_id).expect("Expected repository in config");
        self.installed_storage.add_package(&package, &package_version, source_repository, &install_directory);

        // Download and run post install script if it exists
        let script_name = package_version.get_postinstall_script_name(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;
        if let Some(script_path) = self.download_script("postinstall", &script_name, package_name, &version, &repository_id)? {
            scripts::run_post_script(&script_path, &install_directory, self.config)?;
        }

        Ok(())
    }

    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&mut self, package_name: &str, version: Option<String>) -> Result<()> {
        // Check if the current package to delete is a dependency, if so, give dependency error
        if self.installed_storage.is_dependency(package_name) {
            return Err(InstallerError::DependencyError {
                package_name: package_name.into(),
            });
        }

        // This determines the directory to remove. If there are multiple versions and the version is
        // specified only the specified version directory will be deleted. The entire package directory
        // is deleted if the version isn't specified or if the package directory only contains one version.
        match version {
            Some(version) => self.uninstall_single(package_name, &version)?,
            None => self.uninstall_all(package_name)?,
        };

        Ok(())
    }

    /// Checks if the directory exists. If so, it gets the remove directory for a package version, if there only exists one
    /// version it will return the package directory.
    fn uninstall_single(&mut self, package_name: &str, version: &String) -> Result<()> {
        let installed_versions = self.installed_storage.get_package_versions(package_name);

        // Check if the specified package version exists.
        if !installed_versions.iter().any(|package| package.version == *version) {
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.into(),
                version: version.clone(),
            });
        }

        // Give an error when the user tries to uninstall an external package
        if self.installed_storage.get_package(package_name, &version).expect("Expected package to exist at this point.").external {
            return Err(InstallerError::ExternalError {
                package_name: package_name.into(),
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

        // Remove package from installed package toml
        self.installed_storage.remove_package_version(package_name, &version);

        Ok(())
    }

    // Checks if there exists at least one version of the specified package. If so, it returns the package directory.
    fn uninstall_all(&mut self, package_name: &str) -> Result<()> {
        let installed_versions = self.installed_storage.get_package_versions(package_name);

        // Tell user if one of the package versions is an external package
        for package_version in &installed_versions {
            if package_version.external {
                println!("Packit found external versions of this package, it will only uninstall internal packages.");
                break;
            }
        }

        // Ask the user if he/she wants to continue when version isn't specified and there are multiple versions installed
        let question = "Version is not specified, do you wish to uninstall all versions of this package?";
        if installed_versions.len() > 1 && ask_user(question, QuestionResponse::No)?.is_no_or_invalid() {
            println!("Canceled uninstall of package: {package_name}");
            return Ok(());
        }

        // Make sure at least on version exists
        if installed_versions.is_empty() {
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.into(),
                version: "any".to_string(),
            });
        }

        // Delete the determined directory
        let directory = self.config.install_directory.to_string() + "/" + package_name;
        self.remove_dir_all(&directory, package_name)?;

        // Delete the installed package from toml
        self.installed_storage.remove_package(package_name);

        Ok(())
    }

    /// Wraps around the fs::remove_dir_all to map its error.
    fn remove_dir_all(&self, directory: &str, package_name: &str) -> Result<()> {
        match fs::remove_dir_all(directory) {
            Ok(_) => Ok(()), // TODO: Log succes with display
            Err(e) => Err(InstallerError::UninstallError {
                package_name: package_name.into(),
                e,
            }),
        }
    }

    /// Downloads a script and saves it to the correct directory.
    fn download_script(
        &self,
        script_name: &str,
        script_path: &str,
        package_name: &str,
        version: &str,
        repository_id: &str,
    ) -> Result<Option<String>> {
        let script_destination = format!(
            "{}/{package_name}_{version}_{script_name}.{SCRIPT_EXTENSION}",
            self.config.temp_directory
        );

        match self.repository_manager.read_script(&repository_id, &package_name, &script_path)? {
            Some(script_text) => scripts::save_script(&script_text, &script_destination)?,
            None => return Ok(None), // Script not found, so return None
        }

        // Script succesfully downloaded, so return script location
        Ok(Some(script_destination))
    }
}
