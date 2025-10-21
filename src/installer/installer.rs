use crate::{
    installer::{
        error::{InstallerError, Result},
        unpack::unpack,
    },
    repositories::provider::RepositoryProvider,
    target_architecture::TARGET_ARCHITECTURE,
};

/// The installer of Packit, managing the installation of packages on the system.
pub struct Installer {
    install_directory: String,
}

impl Installer {
    pub fn new(install_directory: String) -> Self {
        Self { install_directory }
    }

    /// Installs the given package and its dependencies.
    pub fn install(&self, provider: &Box<dyn RepositoryProvider>, package_name: &String, version: Option<&String>) -> Result<()> {
        // Use the latest version if the version isn't specified
        let version = match version {
            Some(version) => version,
            None => &provider.read_package(package_name)?.latest_version,
        };

        // Get package info and its target
        let package = provider.read_package_version(package_name, version)?;
        let target = match package.targets.get(TARGET_ARCHITECTURE) {
            Some(target) => target,
            None => return Err(InstallerError::TargetError),
        };

        // Request the data of the package
        let response = match reqwest::blocking::get(&target.url) {
            Ok(response) => response,
            Err(e) => return Err(InstallerError::RequestError(e)),
        };

        // Install global package dependencies and platform specific packages (if there are any, can be empty)
        let dependencies = package.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            self.install(provider, dependency, Option::None)?
        }

        // Install the package in the correct directory
        match target.installer_type.as_str() {
            "unpack" => unpack(response, &self.install_directory)?,
            _ => {}
        }

        Ok(())
    }
}
