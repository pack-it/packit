use serde::{Deserialize, Serialize};

use crate::repositories::types::{Checksum, FileSize};

/// Represents the metadata file that comes with a prebuild.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrebuildFileMeta {
    pub checksum: Checksum,
    pub size: FileSize,
}
