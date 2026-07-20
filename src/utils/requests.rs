// SPDX-License-Identifier: GPL-3.0-only
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
