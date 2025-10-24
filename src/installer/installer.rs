use crate::cli::display::DisplayLoad;
use crate::config::Config;
use crate::installed_packages::InstalledPackageStorage;
use crate::installer::error::Result;
use crate::installer::{error::InstallerError, unpack::unpack};
use crate::repositories::manager::RepositoryManager;
use crate::target_architecture::TARGET_ARCHITECTURE;

/// The installer of Packit, managing the installation of packages on the system.
pub struct Installer {
    install_directory: String,
}

impl Installer {
    pub fn new(install_directory: String) -> Self {
        Self { install_directory }
    }

    /// Installs the given package and its dependencies.
    /// TODO: Maybe move some of the logic
    pub fn install(&self, manager: &RepositoryManager, package_name: &String, version: Option<String>, config: &Config) -> Result<()> {
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

        // Get package version info for its target
        let (repository_id, package_version) = manager.read_package_version(&package_name, &version)?;
        let target = match package_version.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => return Err(InstallerError::TargetError),
        };

        // Install global package dependencies and platform specific packages (if there are any, can be empty)
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            self.install(manager, dependency, Option::None, config)?
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
        display.show_finish("Succesfully downloaded ".to_string() + package_name);

        // Install the package in the correct directory
        match target.installer_type.as_str() {
            "unpack" => {
                unpack(bytes, &self.install_directory)?;
            },
            _ => {},
        }

        // Mark package is installed
        // TODO: Adjust install directory
        let mut installed_storage = InstalledPackageStorage::from(&(self.install_directory.to_string() + "/storage.toml"))?;
        installed_storage.add_package(
            &package,
            &package_version,
            &config.repositories.get(&repository_id).expect("Expected repository in config"),
            &self.install_directory,
        );
        installed_storage.save_to(&(self.install_directory.to_string() + "/storage.toml"))?;

        Ok(())
    }
}
