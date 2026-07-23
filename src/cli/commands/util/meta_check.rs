// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, process::exit};

use clap::Args;
use colored::Colorize;
use url::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, not_found, styled::Styled},
    },
    config::{Config, Repository},
    installer::types::{PackageId, PackageName},
    repositories::{
        error::RepositoryError,
        provider::{self, MetadataProvider},
        types::{Checksum, IndexMeta, Licenses, PackageTarget, PackageVersionMeta, Patch, RepositoryMeta, Source, Sources, TargetBounds},
    },
    utils::{requests, unwrap_or_exit::UnwrapOrExit},
};

/// Checks the metedata of the given package in a repository or all packages in a repository if no package has been given.
#[derive(Args, Debug)]
pub struct MetaCheckArgs {
    /// The repository of the package(s). Can be a repository id specified in `Config.toml`, a path to a repo or a URL to a repo
    repository: String,

    /// The package metadata to check
    package_name: Option<PackageName>,
}

impl HandleCommand for MetaCheckArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit(1);
        let repository = self.get_repository(&config);
        let provider = provider::create_metadata_provider(&repository).unwrap_or_exit_msg("Could not create metadata provider", 1);

        let repository_meta = match provider.read_repository_metadata() {
            Ok(repository_meta) => {
                println!("{}", "Successfully parsed repository metadata".bold().green());
                repository_meta
            },
            Err(e) => {
                error!(e, "Repository metadata could not be parsed");
                exit(1)
            },
        };

        let index = match provider.read_index_metadata() {
            Ok(index_meta) => index_meta,
            Err(e) => {
                error!(e, "Repository 'index.toml' cannot be parsed");
                exit(1);
            },
        };

        let packages = match &self.package_name {
            Some(package) => &HashSet::from([package.clone()]),
            None => &index.supported_packages,
        };

        for package_name in packages {
            self.check_package_meta(&provider, &index, &package_name, &repository_meta);
        }
    }
}

impl MetaCheckArgs {
    fn get_repository(&self, config: &Config) -> Repository {
        if let Some(repository) = config.repositories.get(&self.repository) {
            return repository.clone();
        }

        // Return web repository if the string is a valid URL
        if Url::parse(&self.repository).is_ok() {
            return Repository::new(&self.repository, "web");
        }

        // Return filesystem repository if the string exists as a path
        if matches!(fs::exists(&self.repository), Ok(true)) {
            return Repository::new(&self.repository, "fs");
        }

        error!(msg: "Wrong repository '{}', please use a valid repository id, URL or path to a repository", self.repository);
        exit(1)
    }

    fn check_package_meta(
        &self,
        provider: &Box<dyn MetadataProvider>,
        index: &IndexMeta,
        package_name: &PackageName,
        repository_meta: &RepositoryMeta,
    ) {
        let package_meta = match provider.read_package(&package_name) {
            Ok(package) => {
                let success_message = format!("Successfully parsed {}", package_name.style()).bold().green();
                println!("{success_message}");
                package
            },
            Err(RepositoryError::IOError(..)) => not_found::index_package(package_name, index),
            Err(e) => {
                error!(e, "Package {} could not be parsed", package_name.style());
                return;
            },
        };

        // Check if the package required Packit version is lower then the repository required Packit version
        if let Some(required_version) = &package_meta.required_packit_version
            && repository_meta.required_packit_version >= *required_version
        {
            println!(
                "The required Packit version for {} is lower then or equal to repository '{}' required Packit version",
                package_name.style(),
                self.repository
            );
        }

        if let Some(homepage) = &package_meta.homepage {
            if !requests::check_url(homepage).unwrap_or_exit(1) {
                println!("The homepage URL of {} does not exist", package_name.style());
            }

            // Check if URL is https
            if !homepage.starts_with("https") {
                println!("The homepage URL '{}' of {} is not https", homepage, package_name.style());
            }
        }

        // Check that at least one version is specified
        if package_meta.versions.is_empty() {
            println!("Package {} has no versions listed in its metadata", package_meta.name.style());
        }

        // Check that at least one target bound is specified
        if package_meta.supported_versions.keys().len() == 0 {
            println!("Package {} has no target listed in its metadata", package_meta.name.style());
        }

        // Check that the version intervals for each target are non-empty
        for (target, version_interval) in &package_meta.supported_versions {
            if version_interval.is_empty() {
                println!(
                    "No version interval specified for target '{}' from package {}",
                    target,
                    package_meta.name.style()
                );
            }
        }

        // Check if listed versions exist (cannot be parsed) and do package version specific metadata checks
        for version in &package_meta.versions {
            let package_id = PackageId::new(package_name.clone(), version.clone());
            let package_version = match provider.read_package_version(package_name, &version) {
                Ok(package_version) => {
                    let success_message = format!("Successfully parsed {}", package_id.style()).bold().green();
                    println!("{success_message}");
                    package_version
                },
                Err(e) => {
                    error!(e, "Package {} could not be parsed", package_id.style());
                    continue;
                },
            };

            // Check if the package version required Packit version is lower then the repository required Packit version
            if let Some(required_version) = &package_version.required_packit_version
                && repository_meta.required_packit_version >= *required_version
            {
                println!(
                    "The required Packit version for {} is lower then or equal to the required version in repository '{}'",
                    package_name.style(),
                    self.repository
                );
            }

            // Check if the package version required Packit version is lower then the package required Packit version
            if let Some(package_required_version) = &package_meta.required_packit_version
                && let Some(required_version) = &package_version.required_packit_version
                && package_required_version <= required_version
            {
                println!(
                    "The required Packit version for package {} is lower then or equal to the required version in package version {}",
                    package_name.style(),
                    package_id.style()
                );
            }

            // Check if the version exists in any of the supported ranges
            if !package_meta.supported_versions.values().any(|i| i.covers(version)) {
                println!(
                    "Version {} in {} doesn't exist in any target support range",
                    version.style(),
                    package_name.style()
                )
            }

            self.check_package_version_meta(&package_name, &package_version);
        }
    }

    fn check_package_version_meta(&self, package_name: &PackageName, package_version_meta: &PackageVersionMeta) {
        let package_id = PackageId::new(package_name.clone(), package_version_meta.version.clone());

        // Check license
        self.check_license(&package_version_meta.license, &package_id);

        // Check sources
        let sources = match &package_version_meta.sources {
            Sources::Single(source) => vec![("all", source)],
            Sources::Named(sources) => sources.into_iter().map(|(k, v)| (k.as_str(), v)).collect(),
        };

        // Check if the sources aren't empty
        if sources.is_empty() {
            println!("No sources for package {}", package_id.style());
        }

        // Check all sources
        for (target, source) in sources {
            self.check_source(&package_id, target, source);
        }

        // Check if the targets aren't empty
        if package_version_meta.targets.is_empty() {
            println!("No targets for package {}", package_id.style());
        }

        // Check all targets
        for (bounds, target) in &package_version_meta.targets {
            self.check_target(bounds, target, &package_version_meta.sources);

            // Check if there are duplicates between the package version and target fields
            for dependency in &target.dependencies {
                if package_version_meta.dependencies.iter().any(|d| d.get_name() == dependency.get_name()) {
                    println!("Duplicate dependency '{}' found in {}", dependency, package_id.style());
                }
            }

            for dependency in &target.build_dependencies {
                if package_version_meta.build_dependencies.iter().any(|d| d.get_name() == dependency.get_name()) {
                    println!("Duplicate build dependency '{}' found in {}", dependency, package_id.style());
                }
            }

            if let Some(skip_symlinking) = target.skip_symlinking {
                if package_version_meta.skip_symlinking || !skip_symlinking {
                    println!("Field 'skip_symlinking' unnecessarily specified on target '{}'", bounds);
                }
            }

            for file in &target.external_test_files {
                if package_version_meta.external_test_files.contains(file) {
                    println!("Duplicate external test file '{}' found in {}", file, package_id.style());
                }
            }

            for (key, value) in &target.script_args {
                if let Some(other_value) = package_version_meta.script_args.get(key) {
                    if other_value == value {
                        println!("Duplicate script arg '{} = {}' found in {}", key, value, package_id.style());
                    }
                }
            }
        }
    }

    fn check_source(&self, package_id: &PackageId, target: &str, source: &Source) {
        for url in source.mirrors.iter().chain(std::iter::once(&source.url)) {
            // Check source URL existence
            let response = match requests::get(url) {
                Ok(response) if response.status().is_success() => response,
                _ => {
                    println!("The URL '{}' of {} target '{}' does not exist", url, package_id.style(), target);
                    continue;
                },
            };

            // Check if URL is https
            if !url.starts_with("https") {
                println!("The URL '{}' of {} target '{}' is not https", url, package_id.style(), target);
            }

            // Get bytes from response
            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(e, "Unable to get file bytes");
                    exit(1);
                },
            };

            // Check source checksum
            if source.checksum != Checksum::from_bytes(&bytes) {
                println!(
                    "Checksum '{}' of {} target '{}' with url '{}' is incorrect",
                    source.checksum,
                    package_id.style(),
                    target,
                    url
                );
            }

            // Check source bytes
            if source.size.0 != bytes.len() as u32 {
                println!(
                    "Size '{}' of {} target '{}' with url '{}' is incorrect",
                    source.size,
                    package_id.style(),
                    target,
                    url
                );
            }
        }

        // Check all source patches
        for (patch_number, patch) in &source.patches {
            self.check_patch(package_id, patch_number, patch, target);
        }
    }

    fn check_patch(&self, package_id: &PackageId, patch_number: &u32, patch: &Patch, target: &str) {
        // Check all patch URL's
        for url in patch.mirrors.iter().chain(std::iter::once(&patch.url)) {
            // Check source URL existence
            let response = match requests::get(url) {
                Ok(response) if response.status().is_success() => response,
                _ => {
                    println!(
                        "The URL '{}' of {} target '{}' patch {} does not exist",
                        url,
                        package_id.style(),
                        target,
                        patch_number
                    );
                    continue;
                },
            };

            // Check if URL is https
            if !url.starts_with("https") {
                println!(
                    "The URL '{}' of {} target '{}' patch {} is not https",
                    url,
                    package_id.style(),
                    target,
                    patch_number
                );
            }

            // Get bytes from response
            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(e, "Unable to get file bytes");
                    exit(1);
                },
            };

            // Check source checksum
            if patch.checksum != Checksum::from_bytes(&bytes) {
                println!(
                    "Checksum '{}' of {} target '{}' patch {} with url '{}' is incorrect",
                    patch.checksum,
                    package_id.style(),
                    target,
                    patch_number,
                    url
                );
            }
        }
    }

    fn check_target(&self, bounds: &TargetBounds, target: &PackageTarget, sources: &Sources) {
        // Check if the source reference in the given target is required, or not present when it should be
        match &target.source {
            Some(source_reference) => match &sources {
                Sources::Single(_) => {
                    println!("Found source reference '{source_reference}' for target '{bounds}', eventhough none was required",)
                },
                Sources::Named(sources) if !sources.contains_key(source_reference) => {
                    println!("Source reference '{source_reference}' for target '{bounds}' could not be found in package version metadata",)
                },
                Sources::Named(_) => {},
            },
            None if matches!(sources, Sources::Single(..)) => {},
            None => println!("No source reference found in target, eventhough sources are target specific"),
        }
    }

    fn check_license(&self, license: &Licenses, package_id: &PackageId) {
        let licenses = match &license {
            Licenses::Unknown => return,
            Licenses::Single(license) => &vec![license.clone()],
            Licenses::Any { any } => any,
            Licenses::All { all } => all,
        };

        if licenses.is_empty() {
            println!("License from {} not specified as unknown, but is empty", package_id.style());
        }

        for license in licenses {
            if license.is_empty() {
                println!("Single license is empty in {}", package_id.style())
            }
        }
    }
}
