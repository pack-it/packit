use reqwest::Response;
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};
use tempfile::TempDir;
use thiserror::Error;

use crate::{
    cli::display::Spinner,
    config::Config,
    installer::{
        build_env::BuildEnv,
        scripts::{self, ScriptError},
        unpack::unpack,
    },
    platforms::TARGET_ARCHITECTURE,
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        types::{PackageMetadata, PackageVersion},
    },
    storage::installed_packages::InstalledPackageStorage,
};

/// The errors that occur during building.
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Build files download unsuccessful")]
    RequestUnsuccessful(reqwest::StatusCode),

    #[error("Cannot request files for building")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot unpack response")]
    UnpackError(#[from] std::io::Error),

    #[error("Dependency '{package_name}' of type '{dependency_type}' is not installed.")]
    MissingDependencyError {
        dependency_type: String,
        package_name: String,
    },

    #[error("Cannot execute build script")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot find a repository for building")]
    RepositoryError(#[from] RepositoryError),

    #[error("Checksum does not match")]
    ChecksumError,
}

pub type Result<T> = core::result::Result<T, BuilderError>;

/// The builder of Packit, managing the building of packages.
pub struct Builder<'a> {
    config: &'a Config,
    installed_storage: &'a mut InstalledPackageStorage,
    repository_manager: &'a RepositoryManager<'a>,
}

impl<'a> Builder<'a> {
    /// Creates new builder
    pub fn new(config: &'a Config, installed_storage: &'a mut InstalledPackageStorage, repository_manager: &'a RepositoryManager) -> Self {
        Self {
            config,
            installed_storage,
            repository_manager,
        }
    }

    pub fn build(
        &self,
        package: &PackageMetadata,
        package_version: &PackageVersion,
        repository_id: &str,
        destination_dir: impl AsRef<Path>,
    ) -> Result<()> {
        let target = package_version.get_target(TARGET_ARCHITECTURE)?;

        let mut installed_dependencies = Vec::new();
        let mut installed_build_dependencies = Vec::new();

        // Check if the normal dependencies are installed and get installed package for each dependency.
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            if let Some(package) = self.installed_storage.get_satisfying_package(dependency) {
                installed_dependencies.push(package);

                continue;
            }

            // Return error to indicate the dependency is not installed yet
            return Err(BuilderError::MissingDependencyError {
                dependency_type: "normal".into(),
                package_name: dependency.get_name().into(),
            });
        }

        // Check if the build dependencies are installed and get installed package for each dependency.
        let build_dependencies = package_version.build_dependencies.iter().chain(target.build_dependencies.iter());
        for build_dependency in build_dependencies {
            if let Some(package) = self.installed_storage.get_satisfying_package(build_dependency) {
                installed_build_dependencies.push(package);

                continue;
            }

            // Return error to indicate the dependency is not installed yet
            return Err(BuilderError::MissingDependencyError {
                dependency_type: "build".into(),
                package_name: build_dependency.get_name().into(),
            });
        }

        // Show download spinner
        let spinner = Spinner::new();
        spinner.show("Downloading ".to_string() + &package.name);

        // Download the build files
        let response = reqwest::blocking::get(&target.url)?;
        if !response.status().is_success() {
            return Err(BuilderError::RequestUnsuccessful(response.status()));
        }

        // Get the bytes from the response
        let bytes = response.bytes()?;

        // Calculate the checksum
        let checksum: [u8; 32] = Sha256::digest(&bytes).into();

        // Check equality of checksum
        if target.checksum.sha256 != checksum {
            return Err(BuilderError::ChecksumError);
        }

        // Finish download spinner
        spinner.finish("Downloading ".to_string() + &package.name + " successful");

        // Unpack the package to the temp directory
        let unpack_directory = TempDir::new()?;
        unpack(bytes, &unpack_directory)?;

        // Create build env
        let env = BuildEnv::new(&self.config.prefix_directory, installed_dependencies, installed_build_dependencies);

        // Construct args for the build script
        let script_args = package_version.get_script_args(TARGET_ARCHITECTURE)?;

        // Download and run build script
        let script_path = package_version.get_build_script_path(TARGET_ARCHITECTURE)?;
        let script_path = scripts::download_script(self.repository_manager, &script_path, &package.name, &repository_id)?
            .ok_or(ScriptError::ScriptNotFound("build".into()))?;
        scripts::run_build_script(script_path, &unpack_directory, self.config, &destination_dir, env, &script_args)?;

        Ok(())
    }
}
