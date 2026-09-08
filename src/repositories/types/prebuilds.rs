use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    cli::display::logging::warning,
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

/// Represents the `prebuilds.toml` file, containing a list of all prebuilds that can be generated.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrebuildsList {
    // A mapping from prebuild id to prebuild metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    prebuilds: HashMap<String, PrebuildMeta>,
}

/// Represents the information about a prebuild in the `prebuilds.toml` file.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrebuildMeta {
    targets: Vec<TargetBounds>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_paths: Vec<PathBuf>,
}

impl PrebuildsList {
    /// Gets the prebuild that satisfies the given target the best.
    /// Returns the prebuild id and the `PrebuildMeta`, or `None` if no matching prebuild can be found.
    pub fn get_best_prebuild(&self, target: &Target) -> Option<(&String, &PrebuildMeta)> {
        let mut best_prebuild = None;
        let mut best_priority = 0;

        for (id, meta) in &self.prebuilds {
            let Some((priority, _)) = TargetBounds::get_best_target_priority(target, meta.targets.iter().collect()) else {
                continue;
            };

            if priority < best_priority {
                continue;
            }

            if priority == best_priority {
                warning!("Found two targets that satisfy and have the same priority!");
            }

            best_prebuild = Some((id, meta));
            best_priority = priority;
        }

        best_prebuild
    }

    /// Creates a default `PrebuildsList`, containing a prebuild for all supported targets.
    pub fn default<'a>(supported_targets: impl Iterator<Item = &'a TargetBounds>) -> Self {
        // Extract only supported targets from the given target bounds
        let mut targets = HashSet::new();
        for supported_target in supported_targets {
            match &supported_target.name {
                TargetName::Architecture(architecture) => {
                    targets.insert(architecture);
                },
                TargetName::Os(os) => targets.extend(TargetArchitecture::values().iter().filter(|x| x.get_os() == *os)),
                TargetName::Unix => targets.extend(TargetArchitecture::values().iter().filter(|x| x.get_os().is_unix())),
            }
        }

        let mut prebuilds = HashMap::new();
        for architecture in targets {
            let target = TargetBounds {
                name: TargetName::Architecture(architecture.clone()),
                addition: None,
                version_intervals: VersionIntervals::default(),
            };
            prebuilds.insert(
                architecture.to_string(),
                PrebuildMeta {
                    targets: vec![target],
                    exclude_paths: Vec::new(),
                },
            );
        }

        Self { prebuilds }
    }

    /// Creates the default prebuild id and meta pair for the given target.
    pub fn default_for_target(target: &Target) -> (String, PrebuildMeta) {
        let target_bounds = TargetBounds {
            name: TargetName::Architecture(target.architecture.clone()),
            addition: None,
            version_intervals: VersionIntervals::default(),
        };

        let prebuild_meta = PrebuildMeta {
            targets: vec![target_bounds],
            exclude_paths: Vec::new(),
        };

        (target.architecture.to_string(), prebuild_meta)
    }
}
