use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    installer::types::VersionIntervals,
    platforms::{Target, TargetArchitecture},
    repositories::types::{Checksum, FileSize, TargetBounds, target_bounds::TargetName},
};

/// Represents the metadata file that comes with a prebuild.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrebuildFileMeta {
    pub checksum: Checksum,
    pub size: FileSize,
}

/// Represents the prebuilds.toml file, containing a list of all prebuilds that can be generated.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrebuildsList {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    prebuilds: HashMap<String, PrebuildMeta>,
}

/// Represents the information about a prebuild in the prebuilds.toml file.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrebuildMeta {
    targets: Vec<TargetBounds>,
}

impl PrebuildsList {
    /// Gets the prebuild that satisfies the given target the best.
    /// Returns the prebuild id and the `PrebuildMeta`
    pub fn get_best_prebuild(&self, target: Target) -> (&String, &PrebuildMeta) {
        todo!()
    }
}

// TODO: refactor: also targets that are not supported are in this list by default now.
impl Default for PrebuildsList {
    /// Creates a default `PrebuildsList`, containing a prebuild for all available targets.
    fn default() -> Self {
        let mut prebuilds = HashMap::new();
        for architecture in TargetArchitecture::values() {
            let target = TargetBounds {
                name: TargetName::Architecture(architecture.clone()),
                addition: None,
                version_intervals: VersionIntervals::default(),
            };
            prebuilds.insert(architecture.to_string(), PrebuildMeta { targets: vec![target] });
        }

        Self { prebuilds }
    }
}
