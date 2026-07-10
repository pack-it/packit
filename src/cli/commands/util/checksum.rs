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
        println!("Trying to request file from '{}'", self.url);

        let spinner = Spinner::new("Downloading file".into());
        spinner.show();

        let response = match requests::get(self.url.as_str()) {
            Ok(response) => response,
            Err(e) => {
                error!(e, "Unable to request file");
                exit(1);
            },
        };

        if !response.status().is_success() {
            error!(msg: "File request returned status code {}", response.status().as_u16());
            exit(1);
        }

        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                error!(e, "Unable to get file bytes");
                exit(1);
            },
        };

        let checksum = Checksum::from_bytes(&bytes);

        spinner.finish();
        println!("Checksum: {}", checksum);
        println!("File size: {}", bytes.len());
    }
}
