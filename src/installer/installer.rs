use crate::{
    cli::{ask_user, QuestionResponse, Spinner},
    config::Config,
    installed_packages::InstalledPackageStorage,
    installer::{
        error::{InstallerError, Result},
        scripts::{self, ScriptError, SCRIPT_EXTENSION},
        unpack::unpack,
    },
    platforms::{symlink, TARGET_ARCHITECTURE},
    repositories::manager::RepositoryManager,
};

use std::{fs, path::Path};

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
    pub fn install(&mut self, manager: &RepositoryManager, package_name: &str, version: Option<String>) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        let (repository_id, package) = manager.read_package(package_name)?;

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
        let package_version = manager.read_repo_package_version(&repository_id, &package_name, &version)?;
        let target = match package_version.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => return Err(InstallerError::TargetError),
        };

        // Install global package dependencies and platform specific packages (if there are any, can be empty)
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            self.install(manager, dependency, None)?;
        }

        // Install global package build dependencies and platform specific build dependencies
        let build_dependencies = package_version.build_dependencies.iter().chain(target.build_dependencies.iter());
        for build_dependency in build_dependencies {
            // TODO: Delete build dependencies later, somewhere
            self.install(manager, build_dependency, None)?;
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

        let install_directory = format!("{}/packages/{path_suffix}", self.config.prefix_directory);

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

        // Add and save package to installed storage toml
        let source_repository = self.config.repositories.get(&repository_id).expect("Expected repository in config");
        self.installed_storage.add_package(&package, &package_version, source_repository, &install_directory);

        // Download and run post install script if it exists
        let script_name = &target.postinstall_script;
        if let Some(script_path) = self.download_script("postinstall", script_name, package_name, &version, &repository_id, manager)? {
            scripts::run_post_script(&script_path, &install_directory, self.config)?;
        }

        // Create symlinks for package
        self.create_symlinks(Path::new(&install_directory))?;

        Ok(())
    }

    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&mut self, package_name: &str, version: Option<String>) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

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
            directory = self.config.prefix_directory.to_string() + "/packages/" + package_name;
        } else {
            // The remove directory of a specific package version
            directory = self.config.prefix_directory.to_string() + "/packages/" + package_name + "/" + version;
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
        let directory = self.config.prefix_directory.to_string() + "/packages/" + package_name;
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
        let script_destination = format!("{}/{package_name}_{version}_{script_name}", self.config.temp_directory);
        match manager.read_script(&repository_id, &package_name, &version, &script_name)? {
            Some(script_text) => scripts::save_script(&script_text, &script_destination)?,
            None => return Ok(None), // Script not found, so return None
        }

        // Script succesfully downloaded, so return script location
        Ok(Some(script_destination))
    }

    fn create_symlinks(&self, package_directory: &Path) -> Result<()> {
        let prefix_dir = Path::new(&self.config.prefix_directory);

        // Symlink directories bin, include and lib
        for dir_name in vec!["bin", "include", "lib"] {
            let package_dir_path = package_directory.join(dir_name);
            let prefix_dir_path = prefix_dir.join(dir_name);

            self.create_folder_symlinks(&package_dir_path, &prefix_dir_path)?;
        }

        // Symlink man page directories
        let prefix_man_dir = prefix_dir.join("share").join("man");
        let package_man_dir = package_directory.join("share").join("man");
        if package_man_dir.exists() {
            for man_dir in fs::read_dir(&package_man_dir)? {
                let man_dir = man_dir?;

                let package_dir_path = package_man_dir.join(man_dir.file_name());
                let prefix_dir_path = prefix_man_dir.join(man_dir.file_name());

                self.create_folder_symlinks(&package_dir_path, &prefix_dir_path)?;
            }
        }

        Ok(())
    }

    fn create_folder_symlinks(&self, source_dir: &Path, destination_dir: &Path) -> Result<()> {
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

            // Check if file already exists
            if fs::exists(&destination)? {
                println!("WARNING: symlink {:?} already exists in {:?}", file.file_name(), destination_dir);
                continue;
            }

            // Symlink file in destination directory
            symlink::create_symlink(&file.path(), &destination)?;
        }

        Ok(())
    }

    fn can_write_prefix_dir(&self) -> Result<bool> {
        let metadata = fs::metadata(&self.config.prefix_directory)?;
        let permissions = metadata.permissions();

        Ok(!permissions.readonly())
    }
}
