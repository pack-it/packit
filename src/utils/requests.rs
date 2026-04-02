// SPDX-License-Identifier: GPL-3.0-only
use reqwest::{
    Error, IntoUrl,
    blocking::{Client, Response},
};

/// Sends a `GET` request to the provided url using reqwest.
pub fn get<T: IntoUrl>(url: T) -> Result<Response, Error> {
    build_client()?.get(url).send()
}

/// Builds a request client
fn build_client() -> Result<Client, Error> {
    reqwest::blocking::ClientBuilder::new().user_agent("Packit/0.0.1").build()
}
