use std::collections::HashMap;

use serde::Deserialize;

use crate::repositories::types::Checksum;

/// A script identifier, which holds the scripts name and a bool which specifies
/// if the script should be version specific.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    NameOnly(String),
    Expanded {
        name: String,
        version_specific: bool,
    },
}

/// Holds a URL and mirror URLS to the source code of a package.
/// It also has a checksum to check the validity of the recieved source code.
#[derive(Deserialize, Debug)]
pub struct Source {
    pub url: String,
    pub checksum: Checksum,

    #[serde(default)]
    pub mirrors: Vec<String>,
}

/// Differentiates between Single and Named sources in the toml files.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Sources {
    Single(Source),
    Named(HashMap<String, Source>),
}
