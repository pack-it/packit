use tempfile::TempDir;

use crate::{
    cli::display::{ask_user, display_warning, QuestionResponse, Spinner},
    config::Config,
    installed_packages::InstalledPackageStorage,
    installer::{
        builder::Builder,
        error::{InstallerError, Result},
        scripts,
        types::{Dependency, Version},
    },
    platforms::{symlink, TARGET_ARCHITECTURE},
    repositories::{
        manager::RepositoryManager,
        types::{Package, PackageTarget, PackageVersion},
    },
};

use std::{
    fs,
    path::{Path, PathBuf},
};

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
    pub fn install(&mut self, package_name: &str, version: &Option<Version>) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        let (repository_id, package) = self.repository_manager.read_package(package_name)?;

        // Use the latest version if the version isn't specified
        let version = match version {
            Some(version) => version,
            None => package.latest_versions.get(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?,
        };

        // Check if this package version is already installed
        if self.installed_storage.get_package(package_name, &version).is_some() {
            println!("Package '{} {}' already installed.", package_name, version);
            return Ok(());
        }

        // Get package version info for its target
        let package_version = self.repository_manager.read_repo_package_version(&repository_id, &package_name, &version)?;
        let target = match package_version.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => return Err(InstallerError::TargetError),
        };

        // Install global package dependencies and platform specific packages (if there are any, can be empty)
        self.install_dependencies(&package_version.dependencies, &target.dependencies)?;

        let install_directory = self.config.prefix_directory.join("packages").join(package_name).join(version.to_string());

        // Create install directory if it does not exist
        if !fs::exists(&install_directory)? {
            fs::create_dir_all(&install_directory)?;
        }

        let script_args = package_version.get_script_args(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;

        // Download and run pre install script if it exists
        let script_path = package_version.get_preinstall_script_path(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;
        if let Some(script_file) = scripts::download_script(self.repository_manager, &script_path, package_name, &repository_id)? {
            scripts::run_pre_script(script_file, &install_directory, self.config, &install_directory, &script_args)?;
        }

        let build_destination_dir = TempDir::new()?;

        // Get build version of package
        match self.repository_manager.get_prebuild_url(&repository_id, package_name, version) {
            Some(url) => self.download_prebuild(&url, &build_destination_dir)?,
            None => self.build_package(&package, &package_version, &target, &repository_id, &build_destination_dir)?,
        }

        // Move build to final directory
        fs::rename(build_destination_dir.keep(), &install_directory)?;

        // Check if symlinking should be skipped
        let skip_symlinking = match target.skip_symlinking {
            Some(skip_symlinking) => skip_symlinking,
            None => package_version.skip_symlinking,
        };

        // Add and save package to installed storage toml
        let source_repository = self.config.repositories.get(&repository_id).expect("Expected repository in config");
        self.installed_storage.add_package(&package, &package_version, source_repository, &install_directory, !skip_symlinking);

        // Download and run post install script if it exists
        let script_path = package_version.get_postinstall_script_path(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;
        if let Some(script_file) = scripts::download_script(self.repository_manager, &script_path, package_name, &repository_id)? {
            scripts::run_post_script(script_file, &install_directory, self.config, &script_args)?;
        }

        // Create symlinks for package
        if !skip_symlinking {
            self.create_symlinks(Path::new(&install_directory))?;
        }

        // Download and run test script if it exists
        let script_path = package_version.get_test_script_path(TARGET_ARCHITECTURE).ok_or(InstallerError::TargetError)?;
        if let Some(script_file) = scripts::download_script(self.repository_manager, &script_path, package_name, &repository_id)? {
            scripts::run_test_script(script_file, &install_directory, self.config, &script_args)?;
        }

        Ok(())
    }

    fn install_dependencies<'d>(&mut self, global_dependencies: &Vec<Dependency>, target_dependencies: &Vec<Dependency>) -> Result<()> {
        let dependencies = global_dependencies.iter().chain(target_dependencies.iter());
        for dependency in dependencies {
            if self.installed_storage.dependency_satisfied(dependency) {
                println!("Dependency '{}' already satisfied, continuing", dependency.get_name());
                continue;
            }

            // Determine the latest supported version for the dependency
            let version = self.get_latest_dependency_version(dependency)?;

            self.install(dependency.get_name(), &Some(version.clone()))?;
        }

        Ok(())
    }

    fn build_package(
        &mut self,
        package: &Package,
        package_version: &PackageVersion,
        target: &PackageTarget,
        repository_id: &str,
        destination_dir: impl AsRef<Path>,
    ) -> Result<()> {
        // Install global package build dependencies and platform specific build dependencies
        self.install_dependencies(&package_version.build_dependencies, &target.build_dependencies)?;

        // Build package if we did not find a prebuild
        let builder = Builder::new(self.config, self.installed_storage, self.repository_manager);
        builder.build(&package, &package_version, &repository_id, &destination_dir)?;

        Ok(())
    }

    fn download_prebuild(&self, prebuild_url: &str, destination_dir: impl AsRef<Path>) -> Result<()> {
        todo!()
    }

    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&mut self, package_name: &str, version: &Option<Version>) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        // Check if the current package to delete is a dependency, if so, give dependency error
        if self.installed_storage.is_dependency(package_name, version) {
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
    fn uninstall_single(&mut self, package_name: &str, version: &Version) -> Result<()> {
        let installed_versions = self.installed_storage.get_package_versions(package_name);

        // Check if the specified package version exists.
        if !installed_versions.iter().any(|package| package.version == *version) {
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.into(),
                version: Some(version.to_string()),
            });
        }

        // TODO: refactor this expect
        let installed_package =
            self.installed_storage.get_package(package_name, &version).expect("Expected package to exist at this point.");

        // Return an error when the user tries to uninstall an external package
        if installed_package.external {
            return Err(InstallerError::ExternalError {
                package_name: package_name.into(),
            });
        }

        // Remove entire package directory if there is only one version
        let directory: PathBuf;
        if installed_versions.len() == 1 {
            directory = self.config.prefix_directory.join("packages").join(package_name);
        } else {
            // The remove directory of a specific package version
            directory = self.config.prefix_directory.join("packages").join(package_name).join(version.to_string());
        }

        // Check if the package was symlinked
        if installed_package.symlinked {
            self.remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&directory))?;
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

        // Make sure at least one version exists
        if installed_versions.is_empty() {
            return Err(InstallerError::InstalledExistError {
                package_name: package_name.into(),
                version: None,
            });
        }

        // Path to the determined directory
        let directory = self.config.prefix_directory.join("packages").join(package_name);

        // Check if package was symlinked
        for package_version in &installed_versions {
            if package_version.symlinked {
                self.remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&directory))?;
                break;
            }
        }

        self.remove_dir_all(&directory, package_name)?;

        // Delete the installed package from toml
        self.installed_storage.remove_package(package_name);

        Ok(())
    }

    /// Wraps around the fs::remove_dir_all to map its error.
    fn remove_dir_all(&self, directory: &PathBuf, package_name: &str) -> Result<()> {
        match fs::remove_dir_all(directory) {
            Ok(_) => Ok(()), // TODO: Log succes with display
            Err(e) => Err(InstallerError::UninstallError {
                package_name: package_name.into(),
                e,
            }),
        }
    }

    fn create_symlinks(&self, package_directory: &Path) -> Result<()> {
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
                display_warning!("Symlink {:?} already exists in {:?}", file.file_name(), destination_dir);
                continue;
            }

            // Symlink file in destination directory
            symlink::create_symlink(&file.path(), &destination)?;
        }

        Ok(())
    }

    /// Searches for symlinks with a certain destination (destinations inside of the destination are also a match).
    fn remove_symlinks(&self, search_dir: &Path, destination_dir: &Path) -> Result<()> {
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

    fn can_write_prefix_dir(&self) -> Result<bool> {
        if !fs::exists(&self.config.prefix_directory)? {
            return Ok(false);
        }

        let metadata = fs::metadata(&self.config.prefix_directory)?;
        let permissions = metadata.permissions();

        // TODO: Use something else then readonly, because it can be different for super user and group
        Ok(!permissions.readonly())
    }

    fn get_latest_dependency_version(&self, dependency: &Dependency) -> Result<Version> {
        // Get all supported versions for the dependency
        let (_, package) = self.repository_manager.read_package(&dependency.get_name())?;

        // The supported vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<Version> = None;
        for version in package.versions {
            if !dependency.satisfied(&package.name, Some(&version)) {
                continue;
            }

            current_highest = match current_highest {
                Some(highest) if highest < version => Some(version),
                None => Some(version.clone()),
                _ => continue,
            };
        }

        Ok(current_highest.ok_or(InstallerError::SupportError(dependency.to_string()))?)
    }
}
