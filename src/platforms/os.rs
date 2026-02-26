use std::sync::LazyLock;

use crate::{cli::display::logging::error, installer::types::Version};

#[derive(Clone, Debug)]
pub enum OsVersion {
    Macos {
        version: Version,
    },
    Linux {
        distro: String,
        distro_version: Version,
        kernel_version: Version,
    },
    Windows {
        version: Version,
    },
    Unknown,
}

static CURRENT_VERSION: LazyLock<OsVersion> = LazyLock::new(|| match OsVersion::get_version() {
    Some(value) => value,
    None => OsVersion::Unknown,
});

impl OsVersion {
    pub fn current() -> Self {
        CURRENT_VERSION.clone()
    }

    #[cfg(target_os = "macos")]
    fn get_version() -> Option<Self> {
        use std::{process::Command, str::FromStr};

        let output = match Command::new("/usr/bin/sw_vers").arg("-productVersion").output() {
            Ok(output) => output,
            Err(e) => {
                error!(e, "Cannot fetch macos version");
                return None;
            },
        };

        let version_str = match String::from_utf8(output.stdout) {
            Ok(string) => string,
            Err(e) => {
                error!(e, "Cannot parse macos version to string");
                return None;
            },
        };

        let version = match Version::from_str(version_str.trim()) {
            Ok(version) => version,
            Err(e) => {
                error!(e, "Cannot parse macos version from string");
                return None;
            },
        };

        Some(Self::Macos { version })
    }

    #[cfg(target_os = "linux")]
    fn get_version() -> Option<Self> {
        //TODO
    }

    #[cfg(target_os = "windows")]
    fn get_version() -> Option<Self> {
        //TODO
    }
}
