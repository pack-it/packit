use std::collections::HashMap;

use serde::Deserialize;

use crate::repositories::types::Checksum;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    NameOnly(String),
    Expanded {
        name: String,
        version_specific: bool,
    },
}

#[derive(Deserialize, Debug)]
pub struct Source {
    pub url: String,
    pub checksum: Checksum,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Sources {
    Single(Source),
    Named(HashMap<String, Source>),
}
