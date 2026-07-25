// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, process::exit};

use clap::Args;
use url::Url;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::{Config, Repository},
    installer::types::PackageName,
    integrity::metadata::MetaCheck,
    repositories::provider::{self},
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
        let provider = provider::create_metadata_provider(&repository).unwrap_or_exit_msg("Could not create metadata provider", 1);
        let mut meta_checker = MetaCheck::new(&self.repository, provider);
        meta_checker.check(self.package_name.clone());
        meta_checker.display_issues();
    }
}

impl MetaCheckArgs {
    /// Gets the repository based on the user input (can be a repository id, url or path).
    /// If the repository input is none of these an error is shown and the function exits with code 1.
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
}
