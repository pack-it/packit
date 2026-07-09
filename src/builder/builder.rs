// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;
use colored::Colorize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use url::Url;

use crate::{
    builder::{
        BinaryPatcher, BuildEnv,
        error::{BuilderError, Result},
    },
    cli::display::{Spinner, logging::debug, styled::Styled},
    config::Config,
    installer::{
        install_tree::InstallMeta,
        scripts::{self, ScriptData, ScriptError},
        types::PackageId,
        unpack::{ArchiveExtension, unpack},
    },
    register::package_register::PackageRegister,
    repositories::{
        manager::RepositoryManager,
        types::{Checksum, Patch},
    },
    utils::{ioerror::IOResultExt, patches, requests},
};

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
                package_name: dependency.get_name().clone(),
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
                package_name: build_dependency.get_name().clone(),
            });
        }

        // Get source from the package version
        let source = install_meta.version_metadata.get_source(&install_meta.target_bounds)?;
        debug!("Source size: {}", source.size);

        // Download the build files
        let bytes = self.download_file(&source.url, &source.mirrors, &source.checksum, package_name.style().to_string())?;

        // Create temp directory to build in
        let build_directory = TempDir::new().err_operation("create temp dir")?;

        // Only unpack if the source does not specify the skip_unpack option, write file otherwise
        if !source.skip_unpack {
            let extention = ArchiveExtension::from_path(&source.url);
            unpack(package_name, extention, bytes, &build_directory, true)?;
        } else {
            let url = Url::parse(&source.url)?;
            let file_name = url.path_segments().and_then(|mut x| x.next_back()).ok_or(BuilderError::EmptyUrlPath)?;
            let file_path = build_directory.path().join(file_name);
            fs::write(&file_path, bytes).err_with_path("write", file_path)?;
        }

        // Construct default apply directory for patches
        let mut apply_directory = build_directory.path().to_path_buf();
        if let Some(apply_in) = &source.apply_patches_in {
            apply_directory = apply_directory.join(PathBuf::from(apply_in));
        }

        let package_id = PackageId::new(package_name.clone(), version.clone());

        // Apply patches
        for (id, patch) in source.get_sorted_patches() {
            let patch_bytes = self.download_patch(id, patch, &package_id, &install_meta.repository_id)?;

            // Construct apply directory for this patch
            let apply_directory = match &patch.apply_in {
                Some(apply_in) => &apply_directory.join(PathBuf::from(apply_in)),
                None => &apply_directory,
            };

            // Apply patch
            patches::apply_patch(patch_bytes, &apply_directory)?;

            println!("Applied patch '{id}' to {}", package_id.style());
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
        let script_data = ScriptData::new(&script_path, &destination_dir, &package_id, self.config, &script_args, self.verbose);

        // Show build spinner
        if !self.verbose {
            let styled_package = format!("{package_name}@{version}").bold().blue();
            let spinner_message = format!("Building {styled_package}");
            let spinner = Spinner::new(spinner_message);
            spinner.show();

            // Run build script
            scripts::run_build_script(&script_data, &build_directory, env)?;

            // Finish build spinner
            spinner.finish();
        } else {
            println!("Executing build script of {}", package_id.style());

            // Run build script
            scripts::run_build_script(&script_data, &build_directory, env)?;
        }

        // Patch binaries
        BinaryPatcher::new(self.config).patch_binaries_in(destination_dir.as_ref().to_path_buf(), &package_id, installed_dependencies)?;

        Ok(())
    }

    /// Downloads a patch, either from the given url or from the repository.
    fn download_patch(&self, id: u32, patch: &Patch, package_id: &PackageId, repository_id: &str) -> Result<Bytes> {
        // Download patch from the url if it starts with 'http://' or 'https://'
        if patch.url.starts_with("http://") || patch.url.starts_with("https://") {
            let download_description = format!("patch {id} of {}", package_id.style());
            return self.download_file(&patch.url, &patch.mirrors, &patch.checksum, download_description);
        }

        // Create download spinner
        let spinner_message = format!("Downloading patch {id} of {} from repository '{repository_id}'", package_id.style());
        let spinner = Spinner::new(spinner_message);
        spinner.show();

        // Download patch file from the repository itself
        let file = self
            .repository_manager
            .read_file_bytes(repository_id, &package_id.name, &patch.url)?
            .ok_or(BuilderError::RepositoryPatchNotFound)?;

        // Calculate the checksum
        let calculated_checksum = Checksum::from_bytes(&file);

        // Check equality of checksum
        if patch.checksum != calculated_checksum {
            return Err(BuilderError::ChecksumError);
        }

        // Finish download spinner
        spinner.finish();

        Ok(file)
    }

    /// Downloads a file from the url, or one of the mirrors. Checks against a checksum and shows a spinner.
    fn download_file(&self, url: &str, mirrors: &[String], checksum: &Checksum, download_description: String) -> Result<Bytes> {
        // Show download spinner
        let mut spinner = Spinner::new(format!("Downloading {download_description} from '{}'", &url));
        spinner.show();

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
            spinner.adjust_message(format!("Downloading {download_description} from alternative '{}'", &mirror));

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
        spinner.finish();

        Ok(bytes)
    }
}
