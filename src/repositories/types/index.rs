// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::installer::types::PackageName;

/// An index of all supported packages in the repository
#[derive(Serialize, Deserialize, Debug)]
pub struct IndexMeta {
    pub supported_packages: HashSet<PackageName>,
}
