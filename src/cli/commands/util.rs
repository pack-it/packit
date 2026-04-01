use std::process::exit;

use clap::Subcommand;
use reqwest::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error},
    },
    repositories::types::Checksum,
};

/// Provides several utils for advanced users.
#[derive(Subcommand, Debug)]
pub enum UtilArgs {
    /// Calculates the checksum for the file at the given url
    Checksum {
        /// The url to request the file from
        url: Url,
    },
}

impl HandleCommand for UtilArgs {
    /// Handles the util command.
    fn handle(&self) {
        match self {
            Self::Checksum { url } => self.handle_checksum(url),
        }
    }
}

impl UtilArgs {
    fn handle_checksum(&self, url: &Url) {
        println!("Trying to request file from {url}");

        let spinner = Spinner::new();
        spinner.show("Downloading file".into());

        let response = match reqwest::blocking::get(url.as_str()) {
            Ok(response) => response,
            Err(e) => {
                spinner.stop();
                error!(e, "Unable to request file");
                exit(1);
            },
        };

        if !response.status().is_success() {
            spinner.stop();
            error!(msg: "File request returned status code {}", response.status().as_u16());
            exit(1);
        }

        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                spinner.stop();
                error!(e, "Unable to get file bytes");
                exit(1);
            },
        };

        let checksum = Checksum::from_bytes(&bytes);

        spinner.finish(format!("Found checksum {}", checksum.to_string()));
    }
}
