// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use thiserror::Error;

use crate::{
    config::Config,
    installer::types::{Dependency, PackageId, PackageName},
    packager::{self, PackagerError},
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        types::{IndexMeta, Licenses, PackageVersionMeta, RepositoryMeta},
    },
    utils::{
        ioerror::{self, IOResultExt},
        packit_version::packit_version,
    },
};

const PORTABLE_REPO_MAINTAINER: &str = concat!("Packit v", packit_version!());

/// The errors that occur during portable repository creation.
#[derive(Error, Debug)]
pub enum PortableRepoError {
    #[error("Prebuild for package '{package_id}' cannot be found")]
    PrebuildNotFound {
        package_id: PackageId,
    },

    #[error("Package '{package_name}' has versions in different repositories")]
    PackageFromMultipleRepositories {
        package_name: PackageName,
    },

    #[error("Metadata file of package '{package_name}' cannot be found")]
    FileNotFound {
        package_name: PackageName,
        file_name: String,
    },

    #[error("The destination does already exist and is not an empty directory")]
    DestinationNotEmpty,

    #[error("The given package name is empty")]
    EmptyPackageName,

    #[error("Cannot fetch package from repository")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot create package prebuild")]
    PackagerError(#[from] PackagerError),

    #[error("Cannot serialize toml file")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),
}

pub type Result<T> = std::result::Result<T, PortableRepoError>;

/// The PortableRepoCreator, managing the creation of portable repositories.
pub struct PortableRepoCreator<'a> {
    config: &'a Config,
    repository_manager: &'a RepositoryManager<'a>,
    register: &'a PackageRegister,
    target: Target,
    exclude_prebuilds: bool,
    skip_dependency_resolution: bool,
}

impl<'a> PortableRepoCreator<'a> {
    /// Creates a new PortableRepoCreator.
    pub fn new(
        config: &'a Config,
        repository_manager: &'a RepositoryManager,
        register: &'a PackageRegister,
        target: Target,
        exclude_prebuilds: bool,
        skip_dependency_resolution: bool,
    ) -> Self {
        Self {
            config,
            repository_manager,
            register,
            target,
            exclude_prebuilds,
            skip_dependency_resolution,
        }
    }

    /// Creates a portable repo, containing all given included packages at the given destination.
    pub fn create_portable_repo(&self, included_packages: HashSet<PackageId>, destination: &PathBuf) -> Result<()> {
        // Check if destination is empty
        if destination.exists() && (destination.is_file() || fs::read_dir(destination).err_with_path("read", destination)?.next().is_some())
        {
            return Err(PortableRepoError::DestinationNotEmpty);
        }

        // Create package tree
        let (all_packages, package_index) = self.create_package_tree(included_packages)?;

        for package_id in all_packages {
            let (repository_id, package_version) = self.repository_manager.read_package_version(&package_id, &self.target)?;

            if !self.exclude_prebuilds {
                // Check if the package has a prebuild
                let prebuild_url = self.repository_manager.get_prebuild_url(
                    &repository_id,
                    &package_id,
                    package_version.get_revision_count(),
                    &self.target,
                )?;

                // Check if prebuild is downloadable, or if package is installed
                if prebuild_url.is_none() && (self.target != Target::current() || self.register.get_package_version(&package_id).is_none())
                {
                    return Err(PortableRepoError::PrebuildNotFound { package_id });
                }
            }

            // Download the package metadata
            if !package_index.contains_key(&package_id.name) {
                self.download_package_files(&package_id.name, &repository_id, destination)?;
            }

            // Download the package version files
            self.download_package_version_files(&package_id, &repository_id, &package_version, destination)?;

            // Download prebuild
            if !self.exclude_prebuilds {
                self.download_prebuild(&package_id, &repository_id, &package_version, destination)?;
            }
        }

        // Create repository.toml file
        let repository_meta = RepositoryMeta {
            name: "generated-portable-repo".into(),
            description: format!(
                "A portable repository generated by Packit, containing {} packages",
                package_index.len()
            ),
            maintainers: vec![PORTABLE_REPO_MAINTAINER.into()],
            license: Licenses::Unknown,
            prebuilds_url: None,
            prebuilds_provider: None,
        };
        self.write_metadata(repository_meta, destination.join("repository.toml"), false)?;

        // Create index.toml
        let index_meta = IndexMeta {
            supported_packages: package_index.keys().cloned().collect(),
        };
        self.write_metadata(index_meta, destination.join("index.toml"), true)?;

        Ok(())
    }

    /// Creates a set of all packages that are in the dependency trees of the included packages.
    /// Checks for multiple repository conflicts. Where a conflict is two (or more) versions of the same package that come from different repositories.
    /// Returns a PortableRepoError::PackageFromMultipleRepositories in case of a repository conflict.
    fn create_package_tree(&self, included_packages: HashSet<PackageId>) -> Result<(HashSet<PackageId>, HashMap<PackageName, String>)> {
        let mut all_packages = HashSet::new();
        let mut package_index = HashMap::new();

        let mut package_queue = VecDeque::new();
        for package_id in included_packages {
            package_queue.push_back(package_id);
        }

        while let Some(package_id) = package_queue.pop_front() {
            // Continue if we've already seen the package
            if all_packages.contains(&package_id) {
                continue;
            }

            let package_name = package_id.name.clone();

            let (repository_id, package_version) = self.repository_manager.read_package_version(&package_id, &self.target)?;

            // Check if the package name was previously requested from a different repository
            if package_index.contains_key(&package_name) && package_index[&package_name] != repository_id {
                return Err(PortableRepoError::PackageFromMultipleRepositories { package_name });
            }

            // Add package data to package set and index
            all_packages.insert(package_id);
            package_index.insert(package_name, repository_id);

            // Skip dependency resolution if the flag is enabled
            if self.skip_dependency_resolution {
                continue;
            }

            let target = package_version.get_target(&package_version.get_best_target(&self.target)?)?;

            let dependencies = package_version.dependencies.iter().chain(target.dependencies.iter());
            let dependencies: Vec<&Dependency> = match self.exclude_prebuilds {
                true => dependencies.chain(package_version.build_dependencies.iter()).chain(target.build_dependencies.iter()).collect(),
                false => dependencies.collect(),
            };

            // Add the dependencies to the queue
            for dependency in dependencies {
                let (_, package_metadata) = self.repository_manager.read_package(dependency.get_name())?;
                let version = package_metadata.get_latest_dependency_version(dependency, &self.target)?;
                let dependency_id = PackageId::new(dependency.get_name().clone(), version.clone());
                package_queue.push_back(dependency_id);
            }
        }

        Ok((all_packages, package_index))
    }

    /// Downloads all files of the given package. Note that this does not download version specific files.
    fn download_package_files(&self, package_name: &PackageName, repository_id: &str, destination: &Path) -> Result<()> {
        let package_path = destination.join("packages").join(package_name);

        // Download package.toml
        let package_meta = self.repository_manager.read_repo_package(repository_id, package_name)?;
        self.write_metadata(package_meta, package_path.join("package.toml"), false)?;

        Ok(())
    }

    /// Downloads all files of the given package version.
    fn download_package_version_files(
        &self,
        package_id: &PackageId,
        repository_id: &str,
        package_version: &PackageVersionMeta,
        destination: &Path,
    ) -> Result<()> {
        let package_path = destination.join("packages").join(&package_id.name);

        // Download targets.toml
        let version_meta = self.repository_manager.read_repo_package_version(repository_id, package_id)?;
        let targets_path = package_path.join(package_id.version.to_string()).join("targets.toml");
        self.write_metadata(version_meta, targets_path, false)?;

        let target_bounds = package_version.get_best_target(&self.target)?;

        // Download scripts
        let file_path = package_version.get_build_script_path(&target_bounds)?;
        self.download_file(&package_id.name, repository_id, &file_path, &package_path, false)?;

        let file_path = package_version.get_preinstall_script_path(&target_bounds)?;
        self.download_file(&package_id.name, repository_id, &file_path, &package_path, true)?;

        let file_path = package_version.get_postinstall_script_path(&target_bounds)?;
        self.download_file(&package_id.name, repository_id, &file_path, &package_path, true)?;

        let file_path = package_version.get_test_script_path(&target_bounds)?;
        self.download_file(&package_id.name, repository_id, &file_path, &package_path, true)?;

        let file_path = package_version.get_uninstall_script_path(&target_bounds)?;
        self.download_file(&package_id.name, repository_id, &file_path, &package_path, true)?;

        // Download external test files
        for file_path in package_version.get_external_test_files(&target_bounds)? {
            self.download_file(&package_id.name, repository_id, file_path, &package_path, false)?;
        }

        Ok(())
    }

    /// Downloads the prebuilds of a the given package.
    fn download_prebuild(
        &self,
        package_id: &PackageId,
        repository_id: &str,
        package_version: &PackageVersionMeta,
        destination: &Path,
    ) -> Result<()> {
        let prefix = package_id.name.chars().next().ok_or(PortableRepoError::EmptyPackageName)?.to_string();
        let target = self.target.architecture.to_string();
        let destination = destination.join("prebuilds").join(&target).join(&prefix).join(&package_id.name);
        fs::create_dir_all(&destination).err_with_path("create dirs", &destination)?;

        // Get checksum
        let revision = package_version.get_revision_count();
        let checksum = match self.repository_manager.get_prebuild_checksum(repository_id, package_id, revision, &self.target) {
            Ok(Some(checksum)) => checksum,
            // Only try to package locally if the current target is the target we generate a portable repo for
            Ok(None) if self.target == Target::current() => {
                packager::package(self.config, package_id, &destination, revision)?;
                return Ok(());
            },
            Ok(None) => {
                return Err(PortableRepoError::PrebuildNotFound {
                    package_id: package_id.clone(),
                });
            },
            Err(e) => return Err(e.into()),
        };

        let (_, prebuild) = self.repository_manager.read_prebuild(repository_id, package_id, revision, &self.target)?;

        // Write to file
        let prebuild_name = format!("{package_id}-{revision}-{target}.tar.gz");
        let prebuild_path = destination.join(prebuild_name);
        let checksum_name = format!("{package_id}-{revision}-{target}.sha256");
        let checksum_path = destination.join(checksum_name);
        fs::write(&prebuild_path, &prebuild).err_with_path("write", prebuild_path)?;
        fs::write(&checksum_path, checksum.to_string().as_bytes()).err_with_path("write", checksum_path)?;

        Ok(())
    }

    /// Downloads the file of the given package name from the given repository from the path specified in `file_path`.
    /// The file is saved at `package_path`/`file_path`. Set `allow_missing` to true to skip erroring when the file is missing.
    fn download_file(
        &self,
        package_name: &PackageName,
        repository_id: &str,
        file_path: &str,
        package_path: &Path,
        allow_missing: bool,
    ) -> Result<()> {
        let destination = package_path.join(PathBuf::from(file_path));

        // Skip downloading if we already downloaded the file
        if destination.exists() {
            return Ok(());
        }

        // Read the file from the repository
        let contents = self.repository_manager.read_file(repository_id, package_name, file_path)?;

        // Unwrap the contents or return
        let contents = match contents {
            Some(contents) => contents,
            None if allow_missing => return Ok(()),
            None => {
                return Err(PortableRepoError::FileNotFound {
                    package_name: package_name.clone(),
                    file_name: file_path.into(),
                });
            },
        };

        // Create parent directories
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).err_with_path("create dirs", parent)?;
        }

        // Write file
        fs::write(&destination, contents).err_with_path("write", destination)?;

        Ok(())
    }

    /// Writes the given metadata to the destination file. The pretty option can be used to enable toml pretty printing,
    /// this currently only includes multiline arrays.
    fn write_metadata<M: Serialize>(&self, metadata: M, destination: PathBuf, pretty: bool) -> Result<()> {
        // Create parent directories
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).err_with_path("create dirs", parent)?;
        }

        // Write metadata
        let meta_str = match pretty {
            true => toml::ser::to_string_pretty(&metadata)?,
            false => toml::ser::to_string(&metadata)?,
        };
        fs::write(&destination, meta_str).err_with_path("write", destination)?;

        Ok(())
    }
}
