// SPDX-License-Identifier: GPL-3.0-only
use reqwest::{
    IntoUrl, StatusCode,
    blocking::{Client, Response},
};

use crate::utils::packit_version::packit_version;

const USER_AGENT: &str = concat!("Packit/", packit_version!());

/// Checks if a path escapes the parent directory. Returns true if it does, false if not.
pub fn path_escapes_dir(parent: &str, path: &str) -> bool {
    // Remove prefix and suffix slashes
    let parent = parent.trim_matches('/');
    let path = path.trim_matches('/');

    let mut components = Vec::new();
    for component in path.split('/') {
        if component != ".." {
            components.push(component);
            continue;
        }

        if components.pop().is_none() {
            return true;
        }
    }

    !components.join("/").starts_with(parent)
}

/// Checks if a URL exists by sending a header request.
/// A `GET` method is used as a fallback in case of a head request being blocked.
/// Returns true if the URL exists, false if not.
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

/// Builds a request client.
fn build_client() -> reqwest::Result<Client> {
    reqwest::blocking::ClientBuilder::new().user_agent(USER_AGENT).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_escape() {
        let parent = "/some/parent";
        let path = "/some/parent/foo/child";
        assert!(!path_escapes_dir(parent, path));
        let path = "/some/parent/../parent/foo";
        assert!(!path_escapes_dir(parent, path));
        let path = "/some/parent";
        assert!(!path_escapes_dir(parent, path));

        // Also do some test with different slashes
        let parent = "some/parent/";
        let path = "/some/parent/child/..";
        assert!(!path_escapes_dir(parent, path));
        let path = "/some/other/../parent";
        assert!(!path_escapes_dir(parent, path));
    }

    #[test]
    fn escape() {
        let parent = "/some/parent";
        let path = "/some/parent/..";
        assert!(path_escapes_dir(parent, path));
        let path = "/some/other";
        assert!(path_escapes_dir(parent, path));
        let path = "/some/parent/foo/../../bar";
        assert!(path_escapes_dir(parent, path));
        let path = "/some/parent/../../..";
        assert!(path_escapes_dir(parent, path));
    }

    #[test]
    fn empty_parent_paths() {
        let parent = "";
        let path = "/some/random/path";
        assert!(!path_escapes_dir(parent, path));
        let parent = "/";
        assert!(!path_escapes_dir(parent, path));
    }

    #[test]
    fn empty_child_paths() {
        let parent = "/some/parent";
        let path = "";
        assert!(path_escapes_dir(parent, path));
        let path = "/";
        assert!(path_escapes_dir(parent, path));
    }
}
