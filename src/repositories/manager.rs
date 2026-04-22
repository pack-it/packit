// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use bytes::Bytes;

use crate::{
    cli::display::logging::{debug, error, warning},
    config::Config,
    installer::{
        types::{PackageId, PackageName},
        unpack::ArchiveExtension,
    },
    platforms::Target,
    repositories::{
        error::{RepositoryError, Result},
        provider::{self, MetadataProvider, PrebuildProvider},
        types::{Checksum, IndexMeta, PackageMeta, PackageVersionMeta, RepositoryMeta},
    },
};

/// Manages all requests to the repositories.
pub struct RepositoryManager<'a> {
    config: &'a Config,
    metadata_providers: HashMap<String, Box<dyn MetadataProvider>>,
    prebuild_providers: HashMap<String, Box<dyn PrebuildProvider>>,
}

impl<'a> RepositoryManager<'a> {
    /// Creates a new RepositoryManager.
    pub fn new(config: &'a Config) -> Self {
        let mut metadata_providers = HashMap::new();
        let mut prebuild_providers = HashMap::new();

        for (id, repository) in &config.repositories {
            let provider = provider::create_metadata_provider(repository);
            let Some(provider) = provider else {
                warning!("Cannot create repository provider for repository {id}.");
                continue;
            };

            let repository_meta = match provider.read_repository_metadata() {
                Ok(repository_meta) => Some(repository_meta),
                Err(e) => {
                    error!(e, "Cannot retrieve repository metadata of repository {id}");
                    None
                },
            };

            metadata_providers.insert(id.clone(), provider);

            if let Some(repository_meta) = repository_meta {
                let prebuild_provider = provider::create_prebuild_provider(repository, repository_meta);
                let Some(prebuild_provider) = prebuild_provider else {
                    warning!("Cannot create prebuild provider for repository {id}.");
                    continue;
                };

                prebuild_providers.insert(id.clone(), prebuild_provider);
            }
        }

        Self {
            config,
            metadata_providers,
            prebuild_providers,
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
        for repository_id in &self.config.repositories_rank {
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

            // Check if package contains the target
            if package.get_latest_version(&Target::current()).is_err() {
                continue;
            }

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package.to_string(),
            version: None,
        })
    }

    /// Reads package metadata of the given package from the given repository, containing information about the package.
    /// Does not check if the data contains the current target.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repo_package(&self, repository_id: &str, package: &PackageName) -> Result<PackageMeta> {
        self.get_metadata_provider(repository_id)?.read_package(package)
    }

    /// Reads package version metadata of the given package, containing dependencies and targets.
    /// Returns the id of the repository and the package version metadata.
    pub fn read_package_version(&self, package_id: &PackageId, target: &Target) -> Result<(String, PackageVersionMeta)> {
        for repository_id in &self.config.repositories_rank {
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

            // Check if package contains the target
            if matches!(package.get_best_target(target), Err(RepositoryError::TargetError)) {
                continue;
            }

            // Validate the package before returning
            if package.has_conflicts() {
                return Err(RepositoryError::ValidationError("Package has dependency conflicts.".to_string()));
            }

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package_id.name.to_string(),
            version: Some(package_id.version.to_string()),
        })
    }

    /// Reads package version metadata of the given package from the given repository, containing dependencies and targets.
    /// Does not check if the data contains the current target.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repo_package_version(&self, repository_id: &str, package_id: &PackageId) -> Result<PackageVersionMeta> {
        let provider = self.get_metadata_provider(repository_id)?;

        // Validate the package before returning
        let package = provider.read_package_version(&package_id.name, &package_id.version)?;
        if package.has_conflicts() {
            return Err(RepositoryError::ValidationError("Package has dependency conflicts.".to_string()));
        }

        Ok(package)
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

    /// A helper method to het the metadata provider.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    fn get_metadata_provider(&self, repository_id: &str) -> Result<&Box<dyn MetadataProvider>> {
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
        match self.prebuild_providers.get(repository_id) {
            Some(provider) => Ok(provider),
            None => Err(RepositoryError::RepositoryNotFoundError {
                repository_id: repository_id.into(),
            }),
        }
    }
}
