// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;
use url::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error},
    },
    repositories::types::Checksum,
    utils::requests,
};

/// Calculates the checksum for the file at the given url
#[derive(Args, Debug)]
pub struct ChecksumArgs {
    /// The url to request the file from
    url: Url,
}

impl HandleCommand for ChecksumArgs {
    fn handle(&self) {
        println!("Trying to request file from {}", self.url);

        let spinner = Spinner::new();
        spinner.show("Downloading file".into());

        let response = match requests::get(self.url.as_str()) {
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
        println!("Found checksum {}", checksum);
        println!("Size of file: {}", bytes.len());
    }
}
