use serde::{Deserialize, Serialize};

use crate::installer::types::Version;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub version: Version,
}
