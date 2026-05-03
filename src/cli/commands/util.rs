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
    repositories::{manager::RepositoryManager, portable_repo::PortableRepoCreator, types::Checksum},
    storage::package_register::PackageRegister,
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

    /// Generates a portable repository containing the specified packages
    PortableRepo {
        /// The destination directory
        destination: PathBuf,

        /// The packages to include in the portable repository
        packages: Vec<PackageId>,

        /// True to exclude prebuilds in the portable repository
        #[arg(long, default_value = "false")]
        exclude_prebuilds: bool,
    },
}

impl HandleCommand for UtilArgs {
    /// Handles the util command.
    fn handle(&self) {
        match self {
            Self::Checksum { url } => self.handle_checksum(url),
            Self::PortableRepo {
                destination,
                packages,
                exclude_prebuilds,
            } => self.handle_portable_repo(destination, packages, *exclude_prebuilds),
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

    fn handle_portable_repo(&self, destination: &PathBuf, packages: &Vec<PackageId>, exclude_prebuilds: bool) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register = PackageRegister::from(&PackageRegister::get_default_path(&config)).unwrap_or_exit_msg("Cannot load register", 1);

        let spinner = Spinner::new();
        spinner.show("Generating portable repository".into());

        let creator = PortableRepoCreator::new(&config, &manager, &register, Target::current(), exclude_prebuilds);
        creator
            .create_portable_repo(packages.iter().cloned().collect(), destination)
            .unwrap_or_exit_msg("Cannot create portable repository", 1);

        spinner.finish("Generating portable repository successful".into());
        println!("Created portable repository at {}!", destination.display());
    }
}
