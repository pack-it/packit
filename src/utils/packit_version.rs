// SPDX-License-Identifier: GPL-3.0-only
use std::{process::exit, str::FromStr, sync::LazyLock};

use crate::{cli::display::logging::error, installer::types::Version};

macro_rules! packit_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub(crate) use packit_version;

macro_rules! packit_version_name {
    () => {
        "The Fuzzy Traveling Handyman"
    };
}
pub(crate) use packit_version_name;

/// Global variable keeping the current Packit version.
static PACKIT_VERSION: LazyLock<Version> = LazyLock::new(|| match Version::from_str(packit_version!()) {
    Ok(version) => version,
    Err(e) => {
        error!(e, "Unable to parse current packit version");
        exit(1);
    },
});

/// Gets the current Packit version as Version struct.
pub fn current_packit_version() -> Version {
    PACKIT_VERSION.clone()
}
