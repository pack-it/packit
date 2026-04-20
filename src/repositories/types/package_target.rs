// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::{installer::types::Dependency, repositories::types::Script};

/// Represents the package target data, containing the download url and installer type.
#[derive(Deserialize, Debug)]
pub struct PackageTarget {
    pub source: Option<String>,

    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    #[serde(default)]
    pub build_dependencies: Vec<Dependency>,

    pub skip_symlinking: Option<bool>,

    #[serde(default)]
    pub script_args: HashMap<String, String>,

    pub build_script: Option<Script>,
    pub preinstall_script: Option<Script>,
    pub postinstall_script: Option<Script>,
    pub test_script: Option<Script>,
    pub uninstall_script: Option<Script>,

    #[serde(default)]
    pub external_test_files: HashSet<String>,
}
