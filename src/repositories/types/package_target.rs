// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{installer::types::Dependency, repositories::types::Script};

/// Represents the package target data, containing the download url and installer type.
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageTarget {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_dependencies: Vec<Dependency>,

    pub skip_symlinking: Option<bool>,

    pub source: Option<String>,

    pub build_script: Option<Script>,
    pub preinstall_script: Option<Script>,
    pub postinstall_script: Option<Script>,
    pub test_script: Option<Script>,
    pub uninstall_script: Option<Script>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub external_test_files: HashSet<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub script_args: HashMap<String, String>,
}
