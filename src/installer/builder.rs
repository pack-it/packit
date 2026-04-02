// SPDX-License-Identifier: GPL-3.0-only
use std::path::Path;
use tempfile::TempDir;
use thiserror::Error;

use crate::{
    cli::display::Spinner,
    config::Config,
    installer::{
        build_env::BuildEnv,
        install_tree::InstallMeta,
        scripts::{self, ScriptData, ScriptError},
        types::PackageId,
        unpack::{ArchiveExtension, UnpackError, unpack},
    },
    platforms::binaries::{BinaryPatcher, BinaryPatcherError},
    repositories::{error::RepositoryError, manager::RepositoryManager, types::Checksum},
    storage::package_register::PackageRegister,
    utils::requests,
};

/// The errors that occur during building.
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Build files download unsuccessful, with status code: {0}.")]
    RequestUnsuccessful(reqwest::StatusCode),

    #[error("Dependency '{package_name}' of type '{dependency_type}' is not installed.")]
    MissingDependencyError {
        dependency_type: String,
        package_name: String,
    },

    #[error("Checksum does not match")]
    ChecksumError,

    #[error("Cannot unpack response")]
    UnpackError(#[from] UnpackError),

    #[error("Cannot execute build script")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot find a repository for building")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot patch binaries")]
    PatchError(#[from] BinaryPatcherError),

    #[error("Cannot request files for building")]
    RequestError(#[from] reqwest::Error),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, BuilderError>;

/// The builder of Packit, managing the building of packages.
pub struct Builder<'a> {
    config: &'a Config,
    register: &'a mut PackageRegister,
    repository_manager: &'a RepositoryManager<'a>,
    verbose: bool,
}

impl<'a> Builder<'a> {
    /// Creates new builder
    pub fn new(config: &'a Config, register: &'a mut PackageRegister, repository_manager: &'a RepositoryManager, verbose: bool) -> Self {
        Self {
            config,
            register,
            repository_manager,
            verbose,
        }
    }

    /// Builds a package from the given metadata.
    /// Returns a `BuilderError::MissingDependencyError` if a dependency is missing,
    /// a `BuilderError::RequestUnsuccessful` if a request was unsuccessful or
    /// a `BuilderError::ChecksumError` if the checksums don't match.
    pub fn build(&self, install_meta: &InstallMeta, destination_dir: impl AsRef<Path>) -> Result<()> {
        let package_name = &install_meta.package_metadata.name;
        let version = &install_meta.version_metadata.version;
        let target = install_meta.version_metadata.get_target(&install_meta.target_bounds)?;

        let mut installed_dependencies = Vec::new();
        let mut installed_build_dependencies = Vec::new();

        // Check if the normal dependencies are installed and get installed package for each dependency.
        let dependencies = install_meta.version_metadata.dependencies.iter().chain(target.dependencies.iter());
        for dependency in dependencies {
            if let Some(package) = self.register.get_latest_satisfying_package(dependency) {
                installed_dependencies.push(package);

                continue;
            }

            // Return error to indicate the dependency is not installed yet
            return Err(BuilderError::MissingDependencyError {
                dependency_type: "normal".into(),
                package_name: dependency.get_name().to_string(),
            });
        }

        // Check if the build dependencies are installed and get installed package for each dependency.
        let build_dependencies = install_meta.version_metadata.build_dependencies.iter().chain(target.build_dependencies.iter());
        for build_dependency in build_dependencies {
            if let Some(package) = self.register.get_latest_satisfying_package(build_dependency) {
                installed_build_dependencies.push(package);

                continue;
            }

            // Return error to indicate the dependency is not installed yet
            return Err(BuilderError::MissingDependencyError {
                dependency_type: "build".into(),
                package_name: build_dependency.get_name().to_string(),
            });
        }

        // Get source from the package version
        let source = install_meta.version_metadata.get_source(&install_meta.target_bounds)?;

        // Show download spinner
        let spinner = Spinner::new();
        spinner.show(format!("Downloading '{package_name}' from {}", &source.url));
        let mut finish_message = format!("Downloading '{package_name}' from {} successful", &source.url);

        // Download the build files
        let mut mirrors = source.mirrors.iter();
        let mut response = requests::get(&source.url).map_err(|e| BuilderError::RequestError(e));
        if let Ok(status_response) = &response
            && !status_response.status().is_success()
        {
            response = Err(BuilderError::RequestUnsuccessful(status_response.status()));
        }

        // Loop through mirrors for alternatives in case of error
        while response.is_err()
            && let Some(mirror) = mirrors.next()
        {
            // Update spinner for new download url
            spinner.show(format!("Downloading '{package_name}' from alternative {}", &mirror));
            finish_message = format!("Downloading '{package_name}' from alternative {} successful", &mirror);

            // Get response from alternative mirror
            response = requests::get(mirror).map_err(|e| BuilderError::RequestError(e));

            // Check if the response itself is unsuccessful
            if let Ok(status_response) = &response
                && !status_response.status().is_success()
            {
                response = Err(BuilderError::RequestUnsuccessful(status_response.status()));
            }
        }

        // Get the bytes from the response
        let bytes = response?.bytes()?;

        // Calculate the checksum
        let checksum = Checksum::from_bytes(&bytes);

        // Check equality of checksum
        if source.checksum != checksum {
            return Err(BuilderError::ChecksumError);
        }

        // Finish download spinner
        spinner.finish(finish_message);

        // Unpack the package to the temp directory
        let unpack_directory = TempDir::new()?;
        let extention = ArchiveExtension::from_path(&source.url);
        unpack(package_name, extention, bytes, &unpack_directory, true)?;

        // Create build env
        let env = BuildEnv::new(
            &self.config.prefix_directory,
            &installed_dependencies,
            installed_build_dependencies,
            self.register,
        );

        // Construct args for the build script
        let script_args = install_meta.version_metadata.get_script_args(&install_meta.target_bounds)?;

        // Download and run build script
        let script_path = install_meta.version_metadata.get_build_script_path(&install_meta.target_bounds)?;
        let script_path = scripts::download_script(self.repository_manager, &script_path, &package_name, &install_meta.repository_id)?
            .ok_or(ScriptError::ScriptNotFound("build".into()))?;
        let script_data = ScriptData::new(&script_path, &destination_dir, &version, self.config, &script_args, self.verbose);

        // Show build spinner
        if !self.verbose {
            let spinner = Spinner::new();
            let spinner_message = format!("Building {package_name}@{version}");
            spinner.show(spinner_message.clone());

            // Run build script
            scripts::run_build_script(&script_data, &unpack_directory, env)?;

            // Finish build spinner
            spinner.finish(format!("{spinner_message} successful"));
        } else {
            // Run build script
            scripts::run_build_script(&script_data, &unpack_directory, env)?;
        }

        // Patch binaries
        let package_id = PackageId::new(package_name.clone(), version.clone());
        BinaryPatcher::new(self.config).patch_binaries_in(&destination_dir.as_ref().to_path_buf(), &package_id, installed_dependencies)?;

        Ok(())
    }
}
