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
/// Also has a checksum to check the validity of the received source code.
#[derive(Deserialize, Debug)]
pub struct Source {
    pub url: String,
    pub checksum: Checksum,

    #[serde(default)]
    pub mirrors: Vec<String>,

    #[serde(default)]
    pub skip_unpack: bool,
    pub patches: HashMap<u32, Patch>,
}

/// Wrapper to differentiate between Single and Named sources in the metadata files.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Sources {
    Single(Source),
    Named(HashMap<String, Source>),
}

/// Represents a patch to a source file, holding a URL, mirror URLs and a checksum to check validity.
#[derive(Deserialize, Debug)]
pub struct Patch {
    pub url: String,
    pub checksum: Checksum,
    pub mirrors: Vec<String>,
}

impl Source {
    /// Gets all patches of the source, sorted by id.
    pub fn get_sorted_patches(&self) -> &HashMap<u32, Patch> {
        &self.patches //TODO
    }
}
