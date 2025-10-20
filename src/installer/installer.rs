use crate::installer::error::Result;

use crate::{
    installer::{error::InstallerError, unpack::unpack},
    repositories::provider::RepositoryProvider,
};

pub struct Installer {
    install_directory: String,
}

impl Installer {
    pub fn new(install_directory: String) -> Self {
        Self { install_directory }
    }

    pub fn install(
        &self,
        provider: &impl RepositoryProvider,
        package_name: &String,
        version: Option<&String>,
        platform: &String,
    ) -> Result<()> {
        // Use the latest version if the version isn't specified
        let version = match version {
            Some(version) => version,
            None => &provider.read_package(package_name)?.latest_version,
        };

        let package = provider.read_package_version(package_name.into(), version.into())?;
        let target = match package.targets.get(platform) {
            Some(target) => target,
            None => {
                return Err(InstallerError::TargetError);
            }
        };

        let response = match reqwest::blocking::get(&target.url) {
            Ok(response) => response,
            Err(e) => {
                return Err(InstallerError::RequestError(e));
            }
        };

        // Install package dependencies first
        for dependency in &package.dependencies {
            self.install(provider, dependency, Option::None, platform)?
        }

        for dependency in &target.dependencies {
            self.install(provider, dependency, Option::None, platform)?
        }

        // Install the package in the correct directory
        match target.installer_type.as_str() {
            "unpack" => {
                unpack(response, &self.install_directory)?;
            }
            _ => {}
        }

        Ok(())
    }
}
