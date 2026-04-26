// SPDX-License-Identifier: GPL-3.0-only
use std::{path::PathBuf, process::exit};

use clap::Subcommand;
use reqwest::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error},
    },
    config::Config,
    installer::types::PackageId,
    platforms::Target,
    repositories::{manager::RepositoryManager, minirepo::MiniRepoCreator, types::Checksum},
    utils::{requests, unwrap_or_exit::UnwrapOrExit},
};

/// Provides several utils for advanced users.
#[derive(Subcommand, Debug)]
pub enum UtilArgs {
    /// Calculates the checksum for the file at the given url
    Checksum {
        /// The url to request the file from
        url: Url,
    },

    /// Generates a minirepo containing the specified packages
    Minirepo {
        /// The destination directory
        destination: PathBuf,

        /// The packages to include in the minirepo
        packages: Vec<PackageId>,
    },
}

impl HandleCommand for UtilArgs {
    /// Handles the util command.
    fn handle(&self) {
        match self {
            Self::Checksum { url } => self.handle_checksum(url),
            Self::Minirepo { destination, packages } => self.handle_minirepo(destination, packages),
        }
    }
}

impl UtilArgs {
    fn handle_checksum(&self, url: &Url) {
        println!("Trying to request file from {url}");

        let spinner = Spinner::new();
        spinner.show("Downloading file".into());

        let response = match requests::get(url.as_str()) {
            Ok(response) => response,
            Err(e) => {
                spinner.finish("Downloading file unsuccessful".into());
                error!(e, "Unable to request file");
                exit(1);
            },
        };

        if !response.status().is_success() {
            spinner.finish("Downloading file unsuccessful".into());
            error!(msg: "File request returned status code {}", response.status().as_u16());
            exit(1);
        }

        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                spinner.finish("Downloading file unsuccessful".into());
                error!(e, "Unable to get file bytes");
                exit(1);
            },
        };

        let checksum = Checksum::from_bytes(&bytes);

        spinner.finish("Downloading file successful".into());
        println!("Found checksum {}", checksum.to_string());
    }

    fn handle_minirepo(&self, destination: &PathBuf, packages: &Vec<PackageId>) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);

        let creator = MiniRepoCreator::new(&manager, Target::current());
        creator.create_minirepo(packages.iter().cloned().collect(), destination).unwrap_or_exit_msg("Cannot create minirepo", 1);

        println!("Created minirepo at {}!", destination.display());
    }
}
