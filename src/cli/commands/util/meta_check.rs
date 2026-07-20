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
    repositories::provider,
    utils::unwrap_or_exit::UnwrapOrExit,
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

            for version in package.versions {
                let package_id = PackageId::new(package_name.clone(), version.clone());
                match provider.read_package_version(&package_name, &version) {
                    Ok(_) => {
                        let success_message = format!("Successfully parsed {}", package_id.style()).bold().green();
                        println!("{success_message}")
                    },
                    Err(e) => {
                        error!(e, "Package {} could not be parsed", package_id.style());
                        continue;
                    },
                }
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
}
