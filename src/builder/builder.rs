// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;
use colored::Colorize;
use std::{
    collections::VecDeque,
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
    cli::display::{
        self, ProgressBar, Spinner,
        logging::{debug, warning},
        styled::Styled,
    },
    config::Config,
    installer::{
        install_tree::InstallMeta,
        scripts::{self, ScriptData},
        types::{PackageId, PackageName},
        unpack::{ArchiveExtension, unpack},
    },
    register::package_register::PackageRegister,
    repositories::{
        manager::RepositoryManager,
        types::{Checksum, Patch, Source},
    },
    utils::{io, ioerror::IOResultExt, patches, reading::ReadExt, requests},
};

/// The list of automatically detected license files
const LICENSE_FILE_NAMES: &[&str] = &["license", "licence", "copying", "notice", "copyright"];

/// The list of automatically detected license file extensions
#[rustfmt::skip]
const LICENSE_FILE_EXTENSIONS: &[&str] = &[
    ".txt", ".md", ".markdown", ".mdown", ".mkdn", ".textile", ".rdoc", ".org", ".creole",
    ".mediawiki", ".wiki", ".rst", ".asciidoc", ".adoc", ".asc", ".pod",
];

/// The builder of Packit, managing the building of packages.
pub struct Builder<'a> {
    config: &'a Config,
    register: &'a mut PackageRegister,
    repository_manager: &'a RepositoryManager<'a>,
    verbose: bool,
    execute_build_test: bool,
    pause_build: bool,
}

impl<'a> Builder<'a> {
    /// Creates a new `Builder`.
    pub fn new(
        config: &'a Config,
        register: &'a mut PackageRegister,
        repository_manager: &'a RepositoryManager,
        verbose: bool,
        execute_build_test: bool,
        pause_build: bool,
    ) -> Self {
        Self {
            config,
            register,
            repository_manager,
            verbose,
            execute_build_test,
            pause_build,
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

        // Check if all build requirements are satisfied
        for requirement in &target.build_requirements {
            if !requirement.is_satisfied()? {
                return Err(BuilderError::MissingRequirementError {
                    requirement: requirement.clone(),
                });
            }
        }

        // Get source from the package version
        let source = install_meta.version_metadata.get_source(&install_meta.target_bounds)?;
        debug!("Source size: {}", source.size);

        // Download the build files
        let bytes = self.download_source(source, package_name)?;

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

        // Find inner build directory (a directory with more than just a single directory inside)
        let inner_build_directory = self.find_inner_build_dir(build_directory.path().to_path_buf())?;

        // Construct default apply directory for patches
        let mut apply_directory = inner_build_directory.clone();
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
            patches::apply_patch(patch_bytes, apply_directory)?;

            println!("Applied patch '{id}' to {}", package_id.style());
        }

        // Create build env
        let env = BuildEnv::new(&installed_dependencies, installed_build_dependencies, &target.build_requirements);

        // Construct args for the build script
        let script_args = install_meta.version_metadata.get_script_args(&install_meta.target_bounds)?;

        // Download and run build script
        let script_path = install_meta.version_metadata.get_build_script_path(&install_meta.target_bounds)?;
        let script_path = scripts::download_script(self.repository_manager, &script_path, package_name, &install_meta.repository_id)?;
        let script_data = ScriptData::new(&script_path, &destination_dir, &package_id, self.config, &script_args, self.verbose);

        // Show build spinner
        let script_result;
        if !self.verbose {
            let styled_package = format!("{package_name}@{version}").bold().blue();
            let spinner_message = format!("Building {styled_package}");
            let spinner = Spinner::new(spinner_message);
            spinner.show();

            // Run build script
            script_result = scripts::run_build_script(&script_data, &inner_build_directory, env, self.execute_build_test);

            // Finish build spinner
            spinner.finish();
        } else {
            println!("Executing build script of {}", package_id.style());

            // Run build script
            script_result = scripts::run_build_script(&script_data, &inner_build_directory, env, self.execute_build_test);
        }

        // Wait before continuing when pause build is enabled
        if self.pause_build {
            if let Err(e) = &script_result {
                warning!("Build script execution returned an error: {e}");
            }

            println!("Paused building in '{}'", inner_build_directory.display());
            display::wait_for_continue();
        }

        // Propagate script result
        script_result?;

        // Copy license files
        let license_directory = destination_dir.as_ref().join("share").join("licenses").join(&install_meta.package_metadata.name);
        self.copy_license_files(&inner_build_directory, &license_directory, source)?;

        // Patch binaries
        BinaryPatcher::new(self.config).patch_binaries_in(destination_dir.as_ref().to_path_buf(), &package_id, installed_dependencies)?;

        Ok(())
    }

    // Finds the inner build directory by looking for a directory with more than just a single directory inside.
    // Returns the found inner directory, or the `build_directory` if that is already the inner directory.
    fn find_inner_build_dir(&self, build_directory: PathBuf) -> Result<PathBuf> {
        let mut inner_build_directory = build_directory;
        let mut found_dir = None;

        // Keep searching until we found the absolute inner directory
        loop {
            // Read build directory to see if it contains more than just one directory
            for entry in fs::read_dir(&inner_build_directory).err_with_path("read", &inner_build_directory)? {
                let entry = entry.err_with_path("iterate", &inner_build_directory)?;
                let metadata = entry.metadata().err_with_path("read metadata of", entry.path())?;

                // Found inner directory if the directory contains files
                if !metadata.is_dir() {
                    return Ok(inner_build_directory);
                }

                // Found inner directory if the directory contains more than one directory
                if found_dir.is_some() {
                    return Ok(inner_build_directory);
                }

                // Set found dir to the current subdirectory
                found_dir = Some(entry.path());
            }

            // If we found only one directory, use it as new `inner_build_directory` and search this new directory
            // Otherwise stop searching, inner directory is already found
            match found_dir {
                Some(found_build_dir) => {
                    inner_build_directory = found_build_dir;
                    found_dir = None;
                },
                None => break,
            }
        }

        Ok(inner_build_directory)
    }

    /// Downloads a patch, either from the given url or from the repository. Shows a spinner during the download.
    fn download_patch(&self, id: u32, patch: &Patch, package_id: &PackageId, repository_id: &str) -> Result<Bytes> {
        // Download patch from the url if it starts with 'http://' or 'https://'
        if patch.url.starts_with("http://") || patch.url.starts_with("https://") {
            let spinner_message = format!("Downloading patch {id} of {} from '{}'", package_id.style(), patch.url.cyan());
            let mut spinner = Spinner::new(spinner_message);
            spinner.show();

            let callback = |(alternative, _): (Option<&str>, _)| {
                let Some(alternative) = alternative else { return };

                let message = format!(
                    "Downloading patch {id} of {} from alternative '{}'",
                    package_id.style(),
                    alternative.cyan()
                );
                spinner.adjust_message(message);
            };

            let bytes = self.download_file(&patch.url, &patch.mirrors, &patch.checksum, callback, None)?;
            spinner.finish();
            return Ok(bytes);
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

    /// Downloads the source file of the package. Shows the download progress in a `ProgressBar`.
    fn download_source(&self, source: &Source, package_name: &PackageName) -> Result<Bytes> {
        let retrieve_message = format!("Retrieving {} from '{}'", package_name.style(), source.url.cyan());
        let full_message = format!("{retrieve_message}\nDownloading {}", package_name.style());
        let mut progressbar = ProgressBar::new(source.size.0.into(), full_message);

        let callback = |(alternative, progress): (Option<&str>, Option<usize>)| {
            if let Some(alternative) = alternative {
                let retrieve_message = format!("Retrieving {} from alternative '{}'", package_name.style(), alternative.cyan());
                progressbar.adjust_prefix(format!("{retrieve_message}\nDownloading {}", package_name.style()));
            }

            if let Some(progress) = progress {
                progressbar.set_position(progress as u64);
            }
        };

        let size = source.size.0 as usize;
        self.download_file(&source.url, &source.mirrors, &source.checksum, callback, Some(size))
    }

    /// Downloads a file from the url, or one of the mirrors. Checks against a checksum and returns progress with a callback.
    /// Note that it only returns progress in the callback when `size` is `Some`.
    fn download_file<F>(&self, url: &str, mirrors: &[String], checksum: &Checksum, mut callback: F, size: Option<usize>) -> Result<Bytes>
    where
        F: FnMut((Option<&str>, Option<usize>)),
    {
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
            // Call callback with new download url
            callback((Some(mirror), None));

            // Get response from alternative mirror
            response = requests::get(mirror).map_err(BuilderError::RequestError);

            // Check if the response itself is unsuccessful
            if let Ok(status_response) = &response
                && !status_response.status().is_success()
            {
                response = Err(BuilderError::RequestUnsuccessful(status_response.status()));
            }
        }
        let response = response?;

        // Get the bytes from the response
        let bytes = match size {
            Some(size) => response.read_progress(Some(size), |x| callback((None, Some(x)))).err_operation("read source bytes")?,
            None => response.bytes()?,
        };

        // Calculate the checksum
        let calculated_checksum = Checksum::from_bytes(&bytes);

        // Check equality of checksum
        if *checksum != calculated_checksum {
            return Err(BuilderError::ChecksumError);
        }

        Ok(bytes)
    }

    /// Copies license files from the original source into the destination directory.
    /// Does a breadth-first search from the build directory and stops when it finds license files.
    /// Only traverse to depth 2, to prevent detecting third party license files.
    fn copy_license_files(&self, build_directory: &Path, destination_dir: &Path, source: &Source) -> Result<()> {
        // Copy all include paths first
        self.copy_include_license_files(build_directory, destination_dir, source)?;

        // Skip copying entirely if exclude `*` is specified
        if source.license_exclude.iter().any(|x| x == "*") {
            debug!("Skipping license file copying");
            return Ok(());
        }

        let exclude_paths: Vec<_> = source.license_exclude.iter().map(|x| build_directory.join(x)).collect();

        let mut queue = VecDeque::from([(0, build_directory.to_path_buf())]);
        while let Some((depth, item)) = queue.pop_front() {
            let mut found_files = false;

            // Read all files in the directory
            for entry in fs::read_dir(&item).err_with_path("read", &item)? {
                let entry = entry.err_with_path("iterate", &item)?;

                // Skip paths that should be excluded
                if exclude_paths.contains(&entry.path()) {
                    continue;
                }

                let metadata = entry.metadata().err_with_path("read metadata of", entry.path())?;

                // If the entry is a directory, add it to the queue
                if metadata.is_dir() {
                    // Only add next level if depth is below 2
                    if depth < 2 {
                        queue.push_back((depth + 1, entry.path()));
                    }
                    continue;
                }

                let file_name = entry.file_name().to_ascii_lowercase();
                let Some(file_name) = file_name.to_str() else { continue };

                // Check if file name matches license file names and has the correct extension
                for license_file_name in LICENSE_FILE_NAMES {
                    if file_name.starts_with(license_file_name) {
                        // Skip file if it has an extension and it is not a correct extension
                        if let Some(extension) = io::get_last_extension(file_name)
                            && !LICENSE_FILE_EXTENSIONS.contains(&&*extension.to_lowercase())
                        {
                            break;
                        }

                        found_files = true;
                        debug!(
                            "Found license file at '{}'",
                            entry.path().strip_prefix(build_directory).unwrap_or(build_directory).display()
                        );

                        // Check if file was already copied (for example using `license_include`)
                        let destination_path = destination_dir.join(entry.file_name());
                        if destination_path.exists() {
                            debug!("License file with same name is already copied, skipping file");
                        }

                        // Create destination directory if it does not exist
                        if !destination_dir.exists() {
                            fs::create_dir_all(destination_dir).err_with_path("create dirs", destination_dir)?;
                        }

                        fs::copy(entry.path(), destination_path).err_with_path("copy", entry.path())?;
                        break;
                    }
                }
            }

            // Stop searching if we found files in this directory
            if found_files {
                return Ok(());
            }
        }

        debug!("Unable to find license files for package");
        Ok(())
    }

    /// Copies license files that are listed as `license_include` into the destination directory.
    fn copy_include_license_files(&self, build_directory: &Path, destination_dir: &Path, source: &Source) -> Result<()> {
        for include_path_str in &source.license_include {
            let include_path = build_directory.join(include_path_str);

            // Skip if the path does not exist
            if !include_path.exists() {
                warning!("Specified license file include path '{include_path_str}' does not exist");
                continue;
            }

            // Check if the path is a file
            let metadata = fs::metadata(&include_path).err_with_path("read metadata", &include_path)?;
            if !metadata.is_file() {
                warning!("Specified license file include path '{include_path_str}' is not a file");
                continue;
            }

            let Some(file_name) = include_path.file_name() else {
                warning!("Specified license file include path '{include_path_str}' is not a valid path");
                continue;
            };

            // Create destination directory if it does not exist
            if !destination_dir.exists() {
                fs::create_dir_all(destination_dir).err_with_path("create dirs", destination_dir)?;
            }

            fs::copy(&include_path, destination_dir.join(file_name)).err_with_path("copy", include_path)?;
        }

        Ok(())
    }
}
