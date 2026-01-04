use std::path::Path;

use tempfile::TempDir;
use thiserror::Error;

use crate::{
    cli::Spinner,
    config::Config,
    installed_packages::InstalledPackageStorage,
    installer::{
        scripts::{self, ScriptError},
        unpack::unpack,
    },
    platforms::TARGET_ARCHITECTURE,
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        types::{Package, PackageVersion},
    },
};

/// The errors that occur during building.
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Cannot find target for package.")]
    TargetError,

    #[error("Cannot request files for building: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Cannot unpack response: {0}")]
    UnpackError(#[from] std::io::Error),

    #[error("Dependency '{package_name}' of type '{dependency_type}' is not installed.")]
    MissingDependencyError {
        dependency_type: String,
        package_name: String,
    },

    #[error("Cannot execute script: {0}")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot find a repository for building: {0}")]
    RepositoryError(#[from] RepositoryError),
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
        package: &Package,
        package_version: &PackageVersion,
        repository_id: &str,
        destination_dir: impl AsRef<Path>,
    ) -> Result<()> {
        let target = package_version.targets.get(TARGET_ARCHITECTURE).ok_or(BuilderError::TargetError)?;

        // Check if the normal dependencies are installed
        let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            if self.installed_storage.dependency_satisfied(dependency) {
                continue;
            }

            // Return error to indicate the dependency is not installed yet
            return Err(BuilderError::MissingDependencyError {
                dependency_type: "normal".into(),
                package_name: dependency.get_name().into(),
            });
        }

        // Check if the build dependencies are installed
        let build_dependencies = package_version.build_dependencies.iter().chain(target.build_dependencies.iter());
        for build_dependency in build_dependencies {
            if self.installed_storage.dependency_satisfied(build_dependency) {
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

        // Request the data of the package and get bytes
        let response = reqwest::blocking::get(&target.url)?;
        let bytes = response.bytes()?;

        // Finish download spinner
        spinner.finish("Downloading ".to_string() + &package.name + " successful");

        // Unpack the package to the temp directory
        let unpack_directory = TempDir::new()?;
        dbg!("Created tempdir for unpack: ", unpack_directory.path());
        unpack(bytes, &unpack_directory)?;

        // Construct args for the build script
        let script_args = package_version.get_script_args(TARGET_ARCHITECTURE).ok_or(BuilderError::TargetError)?;

        // Download and run build script
        let script_path = package_version.get_build_script_path(TARGET_ARCHITECTURE).ok_or(BuilderError::TargetError)?;
        let build_script_path = scripts::download_script(self.repository_manager, &script_path, &package.name, &repository_id)?
            .ok_or(ScriptError::ScriptNotFound("build".into()))?;
        scripts::run_build_script(build_script_path, &unpack_directory, self.config, &destination_dir, &script_args)?;

        Ok(())
    }
}
