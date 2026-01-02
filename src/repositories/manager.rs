use std::collections::HashMap;

use crate::{
    cli,
    config::Config,
    platforms::TARGET_ARCHITECTURE,
    repositories::{
        error::{RepositoryError, Result},
        provider::{create_repository_provider, RepositoryProvider},
        types::{Package, PackageVersion, RepositoryMetadata},
    },
    version::Version,
};

/// Manages all requests to the repositories.
pub struct RepositoryManager<'a> {
    config: &'a Config,
    providers: HashMap<String, Box<dyn RepositoryProvider>>,
}

impl<'a> RepositoryManager<'a> {
    /// Creates a new RepositoryManager.
    pub fn new(config: &'a Config) -> Self {
        let mut providers = HashMap::new();

        for (id, repository) in &config.repositories {
            let provider = create_repository_provider(repository);
            if provider.is_none() {
                cli::display_warning(&format!("Cannot create repository provider for repository {id}."));
                continue;
            }

            providers.insert(id.clone(), provider.expect("Expected some provider at this point"));
        }

        Self { config, providers }
    }

    /// Reads repository metadata of the given repository, containing information about the repository.
    pub fn read_repository_metadata(&self, repository_id: &str) -> Result<RepositoryMetadata> {
        let provider = match self.providers.get(repository_id) {
            Some(provider) => provider,
            None => {
                return Err(RepositoryError::RepositoryNotFoundError {
                    repository_id: repository_id.into(),
                })
            },
        };

        Ok(provider.read_repository_metadata()?)
    }

    /// Reads package metadata of the given package, containing information about the package.
    /// Returns the id of the repository and the package metadata.
    pub fn read_package(&self, package: &str) -> Result<(String, Package)> {
        for repository_id in &self.config.repositories_rank {
            let provider = match self.providers.get(repository_id) {
                Some(provider) => provider,
                None => {
                    cli::display_warning(&format!("Cannot find provider for {repository_id}, while it should exist."));
                    continue;
                },
            };

            let package = match provider.read_package(package) {
                Ok(package) => package,
                Err(_) => {
                    println!("Cannot find package {package} in repository {repository_id}, continuing.");
                    continue;
                },
            };

            // Check if package contains the target
            if !package.latest_versions.contains_key(TARGET_ARCHITECTURE) {
                continue;
            }

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package.into(),
            version: None,
        })
    }

    /// Reads package metadata of the given package from the given repository, containing information about the package.
    /// Does not check if the data contains the current target.
    pub fn read_repo_package(&self, repository_id: &str, package: &str) -> Result<Package> {
        let provider = match self.providers.get(repository_id) {
            Some(provider) => provider,
            None => {
                return Err(RepositoryError::RepositoryNotFoundError {
                    repository_id: repository_id.into(),
                })
            },
        };

        Ok(provider.read_package(package)?)
    }

    /// Reads package version metadata of the given package, containing dependencies and targets.
    /// Returns the id of the repository and the package version metadata.
    pub fn read_package_version(&self, package: &str, version: &Version) -> Result<(String, PackageVersion)> {
        for repository_id in &self.config.repositories_rank {
            let provider = match self.providers.get(repository_id) {
                Some(provider) => provider,
                None => {
                    cli::display_warning(&format!("Cannot find provider for {repository_id}, while it should exist."));
                    continue;
                },
            };

            let package = match provider.read_package_version(package, version) {
                Ok(package) => package,
                Err(_) => {
                    println!("Cannot find package {package} {version} in repository {repository_id}, continuing.");
                    continue;
                },
            };

            // Check if package contains the target
            if !package.targets.contains_key(TARGET_ARCHITECTURE) {
                continue;
            }

            return Ok((repository_id.clone(), package));
        }

        Err(RepositoryError::PackageNotFoundError {
            package_name: package.into(),
            version: Some(version.to_string()),
        })
    }

    /// Reads package version metadata of the given package from the given repository, containing dependencies and targets.
    /// Does not check if the data contains the current target.
    pub fn read_repo_package_version(&self, repository_id: &str, package: &str, version: &Version) -> Result<PackageVersion> {
        let provider = match self.providers.get(repository_id) {
            Some(provider) => provider,
            None => {
                return Err(RepositoryError::RepositoryNotFoundError {
                    repository_id: repository_id.into(),
                })
            },
        };

        Ok(provider.read_package_version(package, version)?)
    }

    /// Reads a script of the given package from the given repository.
    /// Returns the script as a string.
    pub fn read_script(&self, repository_id: &str, package: &str, script_path: &str) -> Result<Option<String>> {
        let provider = match self.providers.get(repository_id) {
            Some(provider) => provider,
            None => {
                return Err(RepositoryError::RepositoryNotFoundError {
                    repository_id: repository_id.into(),
                });
            },
        };

        Ok(provider.read_script(package, script_path)?)
    }
}
