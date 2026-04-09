// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use serde::Deserialize;

use crate::repositories::types::Checksum;

/// Represents a script identifier, holding the scripts name and a bool which specifies
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

/// Represents a source, holding a URL and mirror URLs to the source code of a package.
/// Also has a checksum to check the validity of the recieved source code.
#[derive(Deserialize, Debug)]
pub struct Source {
    pub url: String,
    pub checksum: Checksum,

    #[serde(default)]
    pub mirrors: Vec<String>,

    #[serde(default)]
    pub skip_unpack: bool,
}

/// Wrapper to differentiate between Single and Named sources in the metadata files.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Sources {
    Single(Source),
    Named(HashMap<String, Source>),
}
