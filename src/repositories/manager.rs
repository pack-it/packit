// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use bytes::Bytes;

use crate::{
    cli::display::logging::{debug, error, warning},
    config::Config,
    installer::{
        types::{OptionalPackageId, PackageId, PackageName},
        unpack::ArchiveExtension,
    },
    platforms::Target,
    repositories::{
        error::{PackageNotFoundReason, RepositoryError, Result},
        provider::{self, MetadataProvider, PrebuildProvider},
        types::{Checksum, IndexMeta, PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
    utils::packit_version::current_packit_version,
};

/// Manages all requests to the repositories.
pub struct RepositoryManager<'a> {
    config: &'a Config,
    unsupported_repositories: HashMap<String, RepositoryMeta>,
    metadata_providers: HashMap<String, Box<dyn MetadataProvider>>,
    prebuild_providers: HashMap<String, Box<dyn PrebuildProvider>>,
}

impl<'a> RepositoryManager<'a> {
    /// Creates a new RepositoryManager.
    pub fn new(config: &'a Config) -> Self {
        let mut unsupported_repositories = HashMap::new();
        let mut metadata_providers = HashMap::new();
        let mut prebuild_providers = HashMap::new();

        for (id, repository) in &config.repositories {
            let provider = provider::create_metadata_provider(repository);
            let Some(provider) = provider else {
                warning!("Cannot create repository provider for repository {id}.");
                continue;
            };

            // Read repository metadata, or skip repository is metadata cannot be read
            let repository_meta = match provider.read_repository_metadata() {
                Ok(repository_meta) => repository_meta,
                Err(e) => {
                    error!(e, "Cannot retrieve repository metadata of repository {id}, ignoring repository...");
                    continue;
                },
            };

            // Check if the repository works for the current Packit version
            if let Some(required_packit_version) = &repository_meta.required_packit_version {
                if *required_packit_version > current_packit_version() {
                    unsupported_repositories.insert(id.to_string(), repository_meta.clone());
                    warning!("Repository '{id}' requires Packit version {required_packit_version} or higher, ignoring repository...");
                    continue;
                }
            }

            metadata_providers.insert(id.clone(), provider);

            // Try to create the prebuild provider
            let prebuild_provider = provider::create_prebuild_provider(repository, repository_meta);
            let Some(prebuild_provider) = prebuild_provider else {
                warning!("Cannot create prebuild provider for repository {id}.");
                continue;
            };

            prebuild_providers.insert(id.clone(), prebuild_provider);
        }

        Self {
            config,
            unsupported_repositories,
            metadata_providers,
            prebuild_providers,
        }
    }

    /// Reads package and package version metadata of the given package. When the package is only a name, the latest version is read.
    /// Returns the repository id, package metadata and package version metadata.
    /// Returns a `PackageNotFoundError` if the package canot be found.
    pub fn read_package_and_version(
        &self,
        package: &OptionalPackageId,
        target: &Target,
    ) -> Result<(String, PackageMeta, PackageVersionMeta)> {
        match package.versioned() {
            Some(package_id) => {
                // Read package version metadata first, then the according package
                let (repository_id, version_metadata) = self.read_package_version(&package_id, target)?;
                let package_metadata = self.read_repo_package(&repository_id, &package_id.name)?;

                Ok((repository_id, package_metadata, version_metadata))
            },
            None => {
                // Read package metadata first
                let (repository_id, package_metadata) = self.read_package(&package.name)?;

                // Read latest version package version metadata
                let latest_version = package_metadata.get_latest_version(target)?;
                let package_id = PackageId::new(package.name.clone(), latest_version.clone());
                let version_metadata = self.read_repo_package_version(&repository_id, &package_id)?;

                Ok((repository_id, package_metadata, version_metadata))
            },
        }
    }

    /// Reads repository metadata of the given repository, containing information about the repository.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repository_metadata(&self, repository_id: &str) -> Result<RepositoryMeta> {
        self.get_metadata_provider(repository_id)?.read_repository_metadata()
    }

    /// Reads index metadata of the given repository, containing all supported packages.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_index_metadata(&self, repository_id: &str) -> Result<IndexMeta> {
        self.get_metadata_provider(repository_id)?.read_index_metadata()
    }

    /// Reads package metadata of the given package, containing information about the package.
    /// Returns the id of the repository and the package metadata.
    pub fn read_package(&self, package: &PackageName) -> Result<(String, PackageMeta)> {
        let mut not_found_reasons = HashMap::new();

        for repository_id in self.iter_filtered_repositories_rank() {
            let provider = match self.metadata_providers.get(repository_id) {
                Some(provider) => provider,
                None => {
                    warning!("Cannot find provider for {repository_id}, while it should exist.");
                    continue;
                },
            };

            let package = match provider.read_package(package) {
                Ok(package) => package,
                Err(e) => {
                    debug!(err: e, "Unable to read {package} from repository {repository_id}, continuing...");
                    continue;
                },
            };

            // Check package compatibility
            if let Some(reason) = self.check_package_compatibility(&package) {
                not_found_reasons.insert(repository_id, reason);
                continue;
            }

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package.to_string(),
            version: None,
            reason: PackageNotFoundReason::get_primary_reason(not_found_reasons.values()).unwrap_or(PackageNotFoundReason::NotFound),
        })
    }

    /// Reads package metadata of the given package from the given repository, containing information about the package.
    /// Does not check if the data contains the current target.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repo_package(&self, repository_id: &str, package: &PackageName) -> Result<PackageMeta> {
        let package_meta = self.get_metadata_provider(repository_id)?.read_package(package)?;

        if let Some(reason) = self.check_package_compatibility(&package_meta) {
            return Err(RepositoryError::PackageNotFoundError {
                package_name: package.to_string(),
                version: None,
                reason,
            });
        }

        Ok(package_meta)
    }

    /// Checks if a package is compatible.
    /// Returns a Some(PackageNotFoundReason) if the package is not compatible, None otherwise.
    fn check_package_compatibility(&self, package: &PackageMeta) -> Option<PackageNotFoundReason> {
        // Check if package contains the target
        if package.get_latest_version(&Target::current()).is_err() {
            return Some(PackageNotFoundReason::UnsupportedTarget);
        }

        // Check if package is supported by the current Packit version
        if let Some(required_packit_version) = &package.required_packit_version
            && *required_packit_version > current_packit_version()
        {
            return Some(PackageNotFoundReason::NotSupported {
                requires: required_packit_version.clone(),
            });
        }

        None
    }

    /// Reads package version metadata of the given package, containing dependencies and targets.
    /// Returns the id of the repository and the package version metadata.
    pub fn read_package_version(&self, package_id: &PackageId, target: &Target) -> Result<(String, PackageVersionMeta)> {
        let mut not_found_reasons = HashMap::new();

        for repository_id in self.iter_filtered_repositories_rank() {
            let provider = match self.metadata_providers.get(repository_id) {
                Some(provider) => provider,
                None => {
                    warning!("Cannot find provider for {repository_id}, while it should exist.");
                    continue;
                },
            };

            let package = match provider.read_package_version(&package_id.name, &package_id.version) {
                Ok(package) => package,
                Err(_) => {
                    debug!("Cannot find package {package_id} in repository {repository_id}, continuing.");
                    continue;
                },
            };

            // Check package version compatibility
            if let Some(reason) = self.check_package_version_compatibility(&package, target) {
                not_found_reasons.insert(repository_id, reason);
                continue;
            }

            // Validate the package before returning
            if package.has_conflicts() {
                return Err(RepositoryError::ValidationError("Package has conflicts in metadata.".to_string()));
            }

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package_id.name.to_string(),
            version: Some(package_id.version.to_string()),
            reason: PackageNotFoundReason::get_primary_reason(not_found_reasons.values()).unwrap_or(PackageNotFoundReason::NotFound),
        })
    }

    /// Reads package version metadata of the given package from the given repository, containing dependencies and targets.
    /// Does not check if the data contains the current target.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repo_package_version(&self, repository_id: &str, package_id: &PackageId) -> Result<PackageVersionMeta> {
        let provider = self.get_metadata_provider(repository_id)?;
        let package = provider.read_package_version(&package_id.name, &package_id.version)?;

        // Check package version compatibility
        if let Some(reason) = self.check_package_version_compatibility(&package, &Target::current()) {
            return Err(RepositoryError::PackageNotFoundError {
                package_name: package_id.name.to_string(),
                version: Some(package_id.version.to_string()),
                reason,
            });
        }

        // Validate the package before returning
        if package.has_conflicts() {
            return Err(RepositoryError::ValidationError("Package has dependency conflicts.".to_string()));
        }

        Ok(package)
    }

    /// Checks if a package version is compatible.
    /// Returns a Some(PackageNotFoundReason) if the package version is not compatible, None otherwise.
    fn check_package_version_compatibility(&self, package_version: &PackageVersionMeta, target: &Target) -> Option<PackageNotFoundReason> {
        // Check if package contains the target
        if matches!(package_version.get_best_target(target), Err(RepositoryError::TargetError)) {
            return Some(PackageNotFoundReason::UnsupportedTarget);
        }

        None
    }

    /// Reads a file of the given package from the given repository.
    /// Returns the file as bytes.
    pub fn read_file_bytes(&self, repository_id: &str, package: &PackageName, file_path: &str) -> Result<Option<Bytes>> {
        self.get_metadata_provider(repository_id)?.read_file_bytes(package, file_path)
    }

    /// Reads a file of the given package from the given repository.
    /// Returns the file as a string.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_file(&self, repository_id: &str, package: &PackageName, file_path: &str) -> Result<Option<String>> {
        self.get_metadata_provider(repository_id)?.read_file(package, file_path)
    }

    /// Retrieves the prebuild url for the given package version.
    /// Returns the url, or None if a prebuild is not available for the package.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn get_prebuild_url(&self, repository_id: &str, package: &PackageId, revision: u64, target: &Target) -> Result<Option<String>> {
        self.get_prebuid_provider(repository_id)?.get_prebuild_url(package, revision, target)
    }

    /// Retrieves the prebuild checksum for the given package version.
    /// Returns the checksum, or None if a prebuild is not available for the package.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn get_prebuild_checksum(
        &self,
        repository_id: &str,
        package: &PackageId,
        revision: u64,
        target: &Target,
    ) -> Result<Option<Checksum>> {
        self.get_prebuid_provider(repository_id)?.get_prebuild_checksum(package, revision, target)
    }

    /// Reads the prebuild of the given package version as bytes, returns a tuple containing the archive extension and the bytes.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_prebuild(
        &self,
        repository_id: &str,
        package: &PackageId,
        revision: u64,
        target: &Target,
    ) -> Result<(ArchiveExtension, Bytes)> {
        self.get_prebuid_provider(repository_id)?.read_prebuild(package, revision, target)
    }

    /// Iterates over the repositories rank. Filters the unsupported repositories from the list.
    /// Returns an iterator that only contains the ids of working repositories.
    pub fn iter_filtered_repositories_rank(&self) -> impl Iterator<Item = &String> {
        self.config.repositories_rank.iter().filter(|x| !self.unsupported_repositories.contains_key(*x))
    }

    /// Gets the list of repositories that are not supported by this Packit version.
    pub fn get_unsupported_repositories(&self) -> &HashMap<String, RepositoryMeta> {
        &self.unsupported_repositories
    }

    /// A helper method to het the metadata provider.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    fn get_metadata_provider(&self, repository_id: &str) -> Result<&Box<dyn MetadataProvider>> {
        // Check if repository is unsupported
        if self.unsupported_repositories.contains_key(repository_id) {
            return Err(RepositoryError::RepositoryNotSupported {
                repository_id: repository_id.into(),
            });
        }

        match self.metadata_providers.get(repository_id) {
            Some(provider) => Ok(provider),
            None => Err(RepositoryError::RepositoryNotFoundError {
                repository_id: repository_id.into(),
            }),
        }
    }

    /// A helper method to het the prebuild provider.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    fn get_prebuid_provider(&self, repository_id: &str) -> Result<&Box<dyn PrebuildProvider>> {
        // Check if repository is unsupported
        if self.unsupported_repositories.contains_key(repository_id) {
            return Err(RepositoryError::RepositoryNotSupported {
                repository_id: repository_id.into(),
            });
        }

        match self.prebuild_providers.get(repository_id) {
            Some(provider) => Ok(provider),
            None => Err(RepositoryError::RepositoryNotFoundError {
                repository_id: repository_id.into(),
            }),
        }
    }
}
