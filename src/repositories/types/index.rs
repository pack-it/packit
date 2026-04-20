// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashSet;

use serde::Deserialize;

/// An index of all supported packages in the repository
#[derive(Deserialize, Debug)]
pub struct IndexMeta {
    pub supported_packages: HashSet<String>,
}
