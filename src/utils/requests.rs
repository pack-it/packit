// SPDX-License-Identifier: GPL-3.0-only
use reqwest::{
    IntoUrl,
    blocking::{Client, Response},
};

const USER_AGENT: &str = concat!("Packit/", env!("CARGO_PKG_VERSION"));

/// Sends a `GET` request to the provided url using reqwest.
pub fn get<T: IntoUrl>(url: T) -> reqwest::Result<Response> {
    build_client()?.get(url).send()
}

/// Builds a request client
fn build_client() -> reqwest::Result<Client> {
    reqwest::blocking::ClientBuilder::new().user_agent(USER_AGENT).build()
}
