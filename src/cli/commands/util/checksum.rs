// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf, process::exit, str::FromStr};

use clap::Args;
use url::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error},
    },
    repositories::types::Checksum,
    utils::{ioerror::IOResultExt, requests, unwrap_or_exit::UnwrapOrExit},
};

/// Calculates the checksum for the file at the given url.
#[derive(Args, Debug)]
pub struct ChecksumArgs {
    /// The url or path to request the file from
    url: String,
}

impl HandleCommand for ChecksumArgs {
    fn handle(&self) {
        let bytes = match Url::from_str(&self.url) {
            Ok(url) => self.get_bytes_from_url(url),
            Err(_) => self.get_bytes_from_path(&PathBuf::from(self.url.clone())),
        };

        let checksum = Checksum::from_bytes(&bytes);

        println!("Checksum: {}", checksum);
        println!("File size: {}", bytes.len());
    }
}

impl ChecksumArgs {
    /// Gets the bytes from a given url.
    fn get_bytes_from_url(&self, url: Url) -> Vec<u8> {
        let message = format!("Downloading file from {}", self.url);
        let spinner = Spinner::new(message);
        spinner.show();

        let response = match requests::get(url.as_str()) {
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

        spinner.finish();

        match response.bytes() {
            Ok(bytes) => bytes.into(),
            Err(e) => {
                error!(e, "Unable to get file bytes");
                exit(1);
            },
        }
    }

    /// Gets the bytes from a given path.
    fn get_bytes_from_path(&self, path: &PathBuf) -> Vec<u8> {
        fs::read(path).err_with_path("read", path).unwrap_or_exit(1)
    }
}
