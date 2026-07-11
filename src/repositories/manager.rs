// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use bytes::Bytes;

use crate::{
    cli::display::{
        logging::{debug, error, warning},
        styled::Styled,
    },
    config::Config,
    installer::{
        types::{Dependency, OptionalPackageId, PackageId, PackageName, Version},
        unpack::ArchiveExtension,
    },
    platforms::Target,
    repositories::{
        error::{PackageNotFoundReason, RepositoryError, Result},
        provider::{self, MetadataProvider, PrebuildProvider},
        types::{Checksum, Date, IndexMeta, PackageMeta, PackageVersionMeta, RepositoryMeta},
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
    /// Creates a new `RepositoryManager`.
    pub fn new(config: &'a Config) -> Self {
        let mut unsupported_repositories = HashMap::new();
        let mut metadata_providers = HashMap::new();
        let mut prebuild_providers = HashMap::new();

        for (id, repository) in &config.repositories {
            let provider = provider::create_metadata_provider(repository);
            let Some(provider) = provider else {
                warning!("Cannot create repository provider for repository '{id}'.");
                continue;
            };

            // Read repository metadata, or skip repository if the metadata cannot be read
            let repository_meta = match provider.read_repository_metadata() {
                Ok(repository_meta) => repository_meta,
                Err(e) => {
                    error!(
                        e,
                        "Cannot retrieve repository metadata of repository '{id}', ignoring repository..."
                    );
                    continue;
                },
            };

            // Check if the repository works for the current Packit version
            let required_packit_version = &repository_meta.required_packit_version;
            if *required_packit_version > current_packit_version() {
                unsupported_repositories.insert(id.to_string(), repository_meta.clone());
                warning!(
                    "Repository '{id}' requires Packit version {} or higher, ignoring repository...",
                    required_packit_version.style()
                );
                continue;
            }

            metadata_providers.insert(id.clone(), provider);

            // Skip prebuild provider creation if prebuilds are disabled
            if repository.disable_prebuilds {
                continue;
            }

            // Try to create the prebuild provider
            let prebuild_provider = provider::create_prebuild_provider(repository, &repository_meta);
            let Some(prebuild_provider) = prebuild_provider else {
                warning!("Cannot create prebuild provider for repository '{id}'.");
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

    /// Reads package and package version metadata of the given package. When only a package name is given the latest supported version is used.
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
                let version_metadata = self.read_latest_supported_version(&repository_id, &package_metadata, target)?;

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

        for repository_id in self.iter_supported_repositories_rank() {
            let provider = match self.metadata_providers.get(repository_id) {
                Some(provider) => provider,
                None => {
                    warning!("Cannot find provider for '{repository_id}', while it should exist.");
                    continue;
                },
            };

            let package = match provider.read_package(package) {
                Ok(package) => package,
                Err(e) => {
                    debug!(err: e, "Unable to read {} from repository '{repository_id}', continuing...", package.style());
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
            package_name: package.clone(),
            version: None,
            reason: PackageNotFoundReason::get_primary_reason(not_found_reasons.values()),
        })
    }

    /// Reads package metadata of the given package from the given repository, containing information about the package.
    /// Also checks for package compatibility.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repo_package(&self, repository_id: &str, package: &PackageName) -> Result<PackageMeta> {
        let package_meta = self.get_metadata_provider(repository_id)?.read_package(package)?;

        if let Some(reason) = self.check_package_compatibility(&package_meta) {
            return Err(RepositoryError::PackageNotFoundError {
                package_name: package.clone(),
                version: None,
                reason,
            });
        }

        Ok(package_meta)
    }

    /// Checks if a package is compatible, a package is compatible if:
    /// - It's not disabled
    /// - It contains the current target
    /// - The package is supported by the current Packit version
    ///
    /// Returns `Some(PackageNotFoundReason)` if the package is not compatible, `None` otherwise.
    fn check_package_compatibility(&self, package: &PackageMeta) -> Option<PackageNotFoundReason> {
        // Check if the package is disabled
        if let Some(deprecation) = &package.deprecation
            && let Some(disabled_from) = &deprecation.disabled_from
            && *disabled_from <= Date::now()
        {
            return Some(PackageNotFoundReason::Disabled {
                since: disabled_from.clone(),
                reason: deprecation.reason.clone(),
            });
        }

        // Check if package contains the target
        if package.get_supported_versions(&Target::current()).is_err() {
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
    /// Note that this does not check for compatibility of the corresponding `PackageMeta`.
    fn read_package_version(&self, package_id: &PackageId, target: &Target) -> Result<(String, PackageVersionMeta)> {
        let mut not_found_reasons = HashMap::new();

        for repository_id in self.iter_supported_repositories_rank() {
            let provider = match self.metadata_providers.get(repository_id) {
                Some(provider) => provider,
                None => {
                    warning!("Cannot find provider for '{repository_id}', while it should exist.");
                    continue;
                },
            };

            let package = match provider.read_package_version(&package_id.name, &package_id.version) {
                Ok(package) => package,
                Err(_) => {
                    debug!(
                        "Cannot find package {} in repository '{repository_id}', continuing.",
                        package_id.style()
                    );
                    continue;
                },
            };

            // Check package version compatibility
            if let Some(reason) = self.check_package_version_compatibility(&package, target) {
                not_found_reasons.insert(repository_id, reason);
                continue;
            }

            // Validate the package before returning
            package.validate_metadata()?;

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package_id.name.clone(),
            version: Some(package_id.version.clone()),
            reason: PackageNotFoundReason::get_primary_reason(not_found_reasons.values()),
        })
    }

    /// Reads package version metadata of the given package from the given repository, containing dependencies and targets.
    /// Also checks for package version compatibility. Note that this does not check for compatibility of the corresponding `PackageMeta`.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn read_repo_package_version(&self, repository_id: &str, package_id: &PackageId) -> Result<PackageVersionMeta> {
        let provider = self.get_metadata_provider(repository_id)?;
        let package = provider.read_package_version(&package_id.name, &package_id.version)?;

        // Check package version compatibility
        if let Some(reason) = self.check_package_version_compatibility(&package, &Target::current()) {
            return Err(RepositoryError::PackageNotFoundError {
                package_name: package_id.name.clone(),
                version: Some(package_id.version.clone()),
                reason,
            });
        }

        // Validate the package before returning
        package.validate_metadata()?;

        Ok(package)
    }

    /// Checks if a package version is compatible, a package version is compatible if:
    /// - It's not disabled
    /// - It contains the current target
    /// - The package version is supported by the current Packit version
    ///
    /// Returns `Some(PackageNotFoundReason)` if the package version is not compatible, `None` otherwise.
    fn check_package_version_compatibility(&self, package_version: &PackageVersionMeta, target: &Target) -> Option<PackageNotFoundReason> {
        // Check if the package is disabled
        if let Some(deprecation) = &package_version.deprecation
            && let Some(disabled_from) = &deprecation.disabled_from
            && *disabled_from <= Date::now()
        {
            return Some(PackageNotFoundReason::Disabled {
                since: disabled_from.clone(),
                reason: deprecation.reason.clone(),
            });
        }

        // Check if package contains the target
        if matches!(package_version.get_best_target(target), Err(RepositoryError::TargetError)) {
            return Some(PackageNotFoundReason::UnsupportedTarget);
        }

        // Check if package version is supported by the current Packit version
        if let Some(required_packit_version) = &package_version.required_packit_version
            && *required_packit_version > current_packit_version()
        {
            return Some(PackageNotFoundReason::NotSupported {
                requires: required_packit_version.clone(),
            });
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
    /// Returns the url, or `None` if a prebuild is not available for the package.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    pub fn get_prebuild_url(&self, repository_id: &str, package: &PackageId, revision: u64, target: &Target) -> Result<Option<String>> {
        self.get_prebuid_provider(repository_id)?.get_prebuild_url(package, revision, target)
    }

    /// Retrieves the prebuild checksum for the given package version.
    /// Returns the checksum, or `None` if a prebuild is not available for the package.
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

    /// Gets the latest supported package version for the given package metadata.
    pub fn read_latest_supported_version(&self, repository_id: &str, package: &PackageMeta, target: &Target) -> Result<PackageVersionMeta> {
        let supported_versions = package.get_supported_versions(target)?;
        let supported_versions = supported_versions.into_iter().rev();

        self.read_latest_supported_version_impl(repository_id, package, supported_versions)
    }

    /// Gets the latest supported package version for the given package metadata that satisfies the given dependency.
    pub fn read_latest_supported_dependency_version(
        &self,
        repository_id: &str,
        package: &PackageMeta,
        dependency: &Dependency,
        target: &Target,
    ) -> Result<PackageVersionMeta> {
        let supported_versions = package.get_supported_versions(target)?;
        let supported_versions = supported_versions.into_iter().filter(|version| dependency.satisfied(&package.name, version)).rev();

        match self.read_latest_supported_version_impl(repository_id, package, supported_versions) {
            Ok(package_version) => Ok(package_version),
            Err(RepositoryError::PackageNotFoundError { reason, .. }) => Err(RepositoryError::DependencyNotFoundError {
                dependency: dependency.to_string(),
                reason,
            }),
            Err(e) => Err(e),
        }
    }

    /// Implementation of the read latest supported version function.
    /// Resolves the latest supported version based on compatibility checks and implements deprecation resolving.
    /// Note that it assumes a list of supported versions that is in order of low to high.
    fn read_latest_supported_version_impl<'v>(
        &self,
        repository_id: &str,
        package: &PackageMeta,
        supported_versions: impl Iterator<Item = &'v Version>,
    ) -> Result<PackageVersionMeta> {
        let mut reasons = Vec::new();
        let mut latest_deprecated: Option<PackageVersionMeta> = None;
        for latest_version in supported_versions {
            let package_id = PackageId::new(package.name.clone(), latest_version.clone());
            let package_version = match self.read_repo_package_version(repository_id, &package_id) {
                Ok(package_version) => package_version,
                Err(RepositoryError::PackageNotFoundError { reason, .. }) => {
                    reasons.push(reason);
                    continue;
                },
                Err(e) => return Err(e),
            };

            let deprecation = match &package_version.deprecation {
                Some(deprecation) => deprecation,
                None => return Ok(package_version),
            };

            // Update the `latest_deprecated` if it was `None` or if the current version deprecates at a later moment
            match &latest_deprecated {
                Some(latest_deprecation) => {
                    if let Some(latest_deprecation) = &latest_deprecation.deprecation
                        && latest_deprecation.deprecated_from < deprecation.deprecated_from
                    {
                        latest_deprecated = Some(package_version);
                    }
                },
                None => latest_deprecated = Some(package_version),
            }
        }

        // If all versions deprecate, take the latest deprecated version
        if let Some(latest_deprecated) = latest_deprecated {
            return Ok(latest_deprecated);
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package.name.clone(),
            version: None,
            reason: PackageNotFoundReason::get_primary_reason(reasons.iter()),
        })
    }

    /// Iterates over the repositories rank. Filters the unsupported repositories from the list.
    /// Returns an iterator that only contains the ids of supported repositories.
    pub fn iter_supported_repositories_rank(&self) -> impl Iterator<Item = &String> {
        self.config.repositories_rank.iter().filter(|x| !self.unsupported_repositories.contains_key(*x))
    }

    /// Gets a map of repository ids and metadata for all repositories that are not supported by this Packit version.
    pub fn get_unsupported_repositories(&self) -> &HashMap<String, RepositoryMeta> {
        &self.unsupported_repositories
    }

    /// A helper method to get the metadata provider.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    #[expect(clippy::borrowed_box)]
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

    /// A helper method to get the prebuild provider.
    /// Returns a `RepositoryNotFoundError` if no repository with the given `repository_id` can be found.
    #[expect(clippy::borrowed_box)]
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
