// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, process::exit};

use clap::Args;
use colored::Colorize;
use url::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, styled::Styled},
    },
    config::{Config, Repository},
    installer::types::{PackageId, PackageName},
    repositories::{
        provider,
        types::{Checksum, PackageMeta, PackageVersionMeta, Patch, Source, Sources},
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

        // TODO: Better error, telling user which inputs are valid
        let provider = provider::create_metadata_provider(&repository).unwrap_or_exit_msg("Could not create metadata provider", 1);

        let packages = match &self.package_name {
            Some(package) => HashSet::from([package.clone()]),
            None => match provider.read_index_metadata() {
                Ok(index_meta) => index_meta.supported_packages,
                Err(e) => {
                    error!(e, "Repository 'index.toml' cannot be parsed");
                    exit(1);
                },
            },
        };

        match provider.read_repository_metadata() {
            Ok(_) => println!("{}", "Successfully parsed repository metadata".bold().green()),
            Err(e) => error!(e, "Repository metadata could not be parsed"),
        }

        for package_name in packages {
            let package = match provider.read_package(&package_name) {
                Ok(package) => {
                    let success_message = format!("Successfully parsed {}", package_name.style()).bold().green();
                    println!("{success_message}");
                    package
                },
                Err(e) => {
                    error!(e, "Package {} could not be parsed", package_name.style());
                    continue;
                },
            };

            self.check_package_meta(&package);

            for version in package.versions {
                let package_id = PackageId::new(package_name.clone(), version.clone());
                let package_version = match provider.read_package_version(&package_name, &version) {
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

                self.check_package_version_meta(&package_name, &package_version);
            }
        }
    }
}

impl MetaCheckArgs {
    fn get_repository(&self, config: &Config) -> Repository {
        if let Some(repository) = config.repositories.get(&self.repository) {
            return repository.clone();
        }

        let provider_type = match Url::parse(&self.repository) {
            Ok(_) => "web",
            Err(_) => "fs",
        };

        Repository::new(&self.repository, provider_type)
    }

    fn check_package_meta(&self, package_meta: &PackageMeta) {
        if let Some(homepage) = &package_meta.homepage {
            if !requests::check_url(homepage).unwrap_or_exit(1) {
                println!("The homepage URL of {} does not exist", package_meta.name.style());
            }
        }
    }

    fn check_package_version_meta(&self, package_name: &PackageName, package_version_meta: &PackageVersionMeta) {
        let package_id = PackageId::new(package_name.clone(), package_version_meta.version.clone());

        // Check sources
        let sources = match &package_version_meta.sources {
            Sources::Single(source) => vec![("all", source)],
            Sources::Named(sources) => sources.into_iter().map(|(k, v)| (k.as_str(), v)).collect(),
        };

        // Check all sources
        for (target, source) in sources {
            self.check_source(&package_id, target, source);
        }

        // TODO: Check targets
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
}
