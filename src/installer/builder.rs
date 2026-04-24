// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use thiserror::Error;
use url::Url;

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
    utils::{
        patches::{self, PatchError},
        requests,
    },
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

    #[error("The source url has an empty path")]
    EmptyUrlPath,

    #[error("Cannot unpack response")]
    UnpackError(#[from] UnpackError),

    #[error("Cannot execute build script")]
    ScriptError(#[from] ScriptError),

    #[error("Cannot find a repository for building")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot patch binaries")]
    PatchBinaryError(#[from] BinaryPatcherError),

    #[error("Cannot apply patch file")]
    ApplyPatchError(#[from] PatchError),

    #[error("Cannot request files for building")]
    RequestError(#[from] reqwest::Error),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),

    #[error("Cannot parse url of source")]
    UrlParseError(#[from] url::ParseError),
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

        // Download the build files
        let bytes = self.download_file(&source.url, &source.mirrors, &source.checksum, package_name)?;

        // Create temp directory to build in
        let build_directory = TempDir::new()?;

        // Only unpack if the source does not specify the skip_unpack option, write file otherwise
        if !source.skip_unpack {
            let extention = ArchiveExtension::from_path(&source.url);
            unpack(package_name, extention, bytes, &build_directory, true)?;
        } else {
            let url = Url::parse(&source.url)?;
            let file_name = url.path_segments().and_then(|mut x| x.next_back()).ok_or(BuilderError::EmptyUrlPath)?;
            let file_path = build_directory.path().join(file_name);
            fs::write(file_path, bytes)?;
        }

        // Construct default apply directory for patches
        let mut apply_directory = build_directory.path().to_path_buf();
        if let Some(apply_in) = &source.apply_patches_in {
            apply_directory = apply_directory.join(PathBuf::from(apply_in));
        }

        // Apply patches
        for (id, patch) in source.get_sorted_patches() {
            let description = format!("patch {id}' of '{package_name}");
            let patch_bytes = self.download_file(&patch.url, &patch.mirrors, &patch.checksum, &description)?;

            // Construct apply directory for this patch
            let apply_directory = match &patch.apply_in {
                Some(apply_in) => apply_directory.join(PathBuf::from(apply_in)),
                None => apply_directory.clone(),
            };

            // Apply patch
            patches::apply_patch(&patch_bytes, &apply_directory)?;

            println!("Applied patch '{id}' to '{package_name}'");
        }

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
        let script_path = scripts::download_script(self.repository_manager, &script_path, package_name, &install_meta.repository_id)?
            .ok_or(ScriptError::ScriptNotFound("build".into()))?;
        let package_id = PackageId::new(package_name.clone(), version.clone());
        let script_data = ScriptData::new(&script_path, &destination_dir, &package_id, self.config, &script_args, self.verbose);

        let package_id = PackageId::new(package_name.clone(), version.clone());
        println!("Executing build script of {package_id}");

        // Show build spinner
        if !self.verbose {
            let spinner = Spinner::new();
            let spinner_message = format!("Building {package_name}@{version}");
            spinner.show(spinner_message.clone());

            // Run build script
            scripts::run_build_script(&script_data, &build_directory, env)?;

            // Finish build spinner
            spinner.finish(format!("{spinner_message} successful"));
        } else {
            // Run build script
            scripts::run_build_script(&script_data, &build_directory, env)?;
        }

        // Patch binaries
        BinaryPatcher::new(self.config).patch_binaries_in(&destination_dir.as_ref().to_path_buf(), &package_id, installed_dependencies)?;

        Ok(())
    }

    /// Downloads a file from the url, or one of the mirrors. Checks against a checksum and shows a spinner.
    fn download_file(&self, url: &str, mirrors: &Vec<String>, checksum: &Checksum, download_description: &str) -> Result<Bytes> {
        // Show download spinner
        let spinner = Spinner::new();
        spinner.show(format!("Downloading '{download_description}' from {}", &url));
        let mut finish_message = format!("Downloading '{download_description}' from {} successful", &url);

        // Try to download from the main url
        let mut mirrors = mirrors.iter();
        let mut response = requests::get(url).map_err(BuilderError::RequestError);
        if let Ok(status_response) = &response
            && !status_response.status().is_success()
        {
            response = Err(BuilderError::RequestUnsuccessful(status_response.status()));
        }

        // Loop through mirrors for alternatives in case of error
        while response.is_err()
            && let Some(mirror) = mirrors.next()
        {
            // Update spinner with new download url
            spinner.show(format!("Downloading '{download_description}' from alternative {}", &mirror));
            finish_message = format!("Downloading '{download_description}' from alternative {} successful", &mirror);

            // Get response from alternative mirror
            response = requests::get(mirror).map_err(BuilderError::RequestError);

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
        let calculated_checksum = Checksum::from_bytes(&bytes);

        // Check equality of checksum
        if *checksum != calculated_checksum {
            return Err(BuilderError::ChecksumError);
        }

        // Finish download spinner
        spinner.finish(finish_message);

        Ok(bytes)
    }
}
