// SPDX-License-Identifier: GPL-3.0-only
use std::io::Read;

use bytes::{Bytes, BytesMut};
use reqwest::{
    IntoUrl, StatusCode,
    blocking::{Client, Response},
};

use crate::utils::packit_version::packit_version;

const USER_AGENT: &str = concat!("Packit/", packit_version!());

/// Checks if a URL exists by sending a header request.
/// A `GET` method is used as a fallback in case of a head request being blocked.
/// Returns true if the URL exists, false if not
pub fn check_url<T: IntoUrl + Clone>(url: T) -> reqwest::Result<bool> {
    let client = build_client()?;

    // Send a request header, have a `GET` method as fallback in case the head request is blocked
    let response = match client.head(url.clone()).send() {
        Ok(response) if response.status() != StatusCode::METHOD_NOT_ALLOWED => response,
        _ => match client.get(url).send() {
            Ok(response) => response,
            Err(_) => return Ok(false),
        },
    };

    Ok(response.status().is_success())
}

/// Sends a `GET` request to the provided url using reqwest.
pub fn get<T: IntoUrl>(url: T) -> reqwest::Result<Response> {
    build_client()?.get(url).send()
}

/// Builds a request client
fn build_client() -> reqwest::Result<Client> {
    reqwest::blocking::ClientBuilder::new().user_agent(USER_AGENT).build()
}

pub trait ResponseExt {
    /// Reads all bytes from the response into a byte buffer.
    /// Calls the `progress` callback after reading a chunk.
    fn read_all<F>(self, progress: F) -> std::io::Result<Bytes>
    where
        F: FnMut(usize);
}

impl ResponseExt for Response {
    fn read_all<F>(mut self, mut progress: F) -> std::io::Result<Bytes>
    where
        F: FnMut(usize),
    {
        // Use content length, but cap at 100MB. Use 1MB if no size is specified
        let size = self.content_length().map(|x| x.min(100 * 1024 * 1024)).unwrap_or(1 * 1024 * 1024) as usize;
        let mut bytes = BytesMut::with_capacity(size);
        let mut buffer = [0; 32 * 1024];

        loop {
            // Read response into buffer, retry on interrupted
            let n = match self.read(&mut buffer) {
                Ok(n) => n,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::Interrupted) => continue,
                Err(e) => return Err(e),
            };

            // Stop reading if end of stream is reached
            if n == 0 {
                break;
            }

            // Add read buffer to final buffer
            bytes.extend_from_slice(&buffer[..n]);

            // Call progress callback
            progress(bytes.len());
        }

        Ok(bytes.freeze())
    }
}
