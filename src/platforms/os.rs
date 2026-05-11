// SPDX-License-Identifier: GPL-3.0-only
use std::sync::LazyLock;

use crate::{cli::display::logging::debug, installer::types::Version};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::str::FromStr;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use crate::cli::display::logging::error;

/// Represents an operating system type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Os {
    MacOs,
    Linux,
    Windows,
    Unknown,
}

impl Os {
    /// Checks if the current OS is Unix based.
    pub fn is_unix(&self) -> bool {
        matches!(self, Self::MacOs | Self::Linux)
    }

    #[cfg(target_os = "macos")]
    pub fn current() -> Self {
        Self::MacOs
    }

    #[cfg(target_os = "linux")]
    pub fn current() -> Self {
        Self::Linux
    }

    #[cfg(target_os = "windows")]
    pub fn current() -> Self {
        Self::Windows
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    pub fn current() -> Self {
        Self::Unknown
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MacOs => "mac",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Unknown => "unknown",
        }
    }
}

/// Represents an OS version. In case of Linux this also includes the distro and distro version.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum OsVersion {
    MacOs {
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

/// Global variable keeping the current OS version.
static CURRENT_VERSION: LazyLock<OsVersion> = LazyLock::new(|| match OsVersion::get_version() {
    Some(value) => value,
    None => OsVersion::Unknown,
});

impl OsVersion {
    /// Returns the OsVersion of the current system.
    pub fn current() -> Self {
        CURRENT_VERSION.clone()
    }

    /// Gets the macOS version.
    /// Returns None if the version cannot be fetched or parsed.
    #[cfg(target_os = "macos")]
    fn get_version() -> Option<Self> {
        use std::process::Command;

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

        debug!("Retrieved current macOS version {version}");

        Some(Self::MacOs { version })
    }

    /// Gets the Linux version, represented with the distro name, distro version and kernel version.
    /// Returns None if version information cannot be read or parsed, although an error message might sometimes be shown.
    #[cfg(target_os = "linux")]
    fn get_version() -> Option<Self> {
        use std::fs;

        let kernel_version_str = match fs::read_to_string("/proc/sys/kernel/osrelease") {
            Ok(info) => info,
            Err(e) => {
                error!(e, "Cannot read kernel version");
                return None;
            },
        };

        let kernel_version_str = match kernel_version_str.split_once("-") {
            Some((version, _)) => version,
            None => &kernel_version_str,
        };

        let kernel_version = match Version::from_str(kernel_version_str.trim()) {
            Ok(version) => version,
            Err(e) => {
                error!(e, "Cannot parse kernel version from string");
                return None;
            },
        };

        debug!("Retrieved current kernel version {kernel_version}");

        let distro_info = match fs::read_to_string("/etc/os-release") {
            Ok(info) => info,
            Err(e) => {
                error!(e, "Cannot read distro information");
                return None;
            },
        };

        let mut distro = None;
        let mut distro_version = None;
        for line in distro_info.split("\n") {
            if line.trim().is_empty() {
                continue;
            }

            let (key, value) = match line.split_once("=") {
                Some(val) => val,
                None => {
                    error!(msg: "Invalid result from /etc/os-release: line does not contain '='");
                    continue;
                },
            };

            // Remove quotes from value
            let value = match value.starts_with("\"") && value.ends_with("\"") {
                true => &value[1..value.len() - 1],
                false => value,
            };

            match key.to_lowercase().as_str() {
                "id" => distro = Some(value),
                "version_id" => distro_version = Some(value),
                _ => continue,
            }
        }

        let distro = match distro {
            Some(distro) => distro,
            None => {
                error!(msg: "Cannot read distro name");
                return None;
            },
        };

        let distro_version = match distro_version {
            Some(version) => match Version::from_str(version.trim()) {
                Ok(version) => version,
                Err(e) => {
                    error!(e, "Cannot parse distro version from string");
                    return None;
                },
            },
            None => {
                error!(msg: "Cannot read distro version");
                return None;
            },
        };

        debug!("Retrieved current distro {distro} with version {distro_version}");

        Some(Self::Linux {
            distro: distro.into(),
            distro_version,
            kernel_version,
        })
    }

    /// Gets the current Windows version.
    #[cfg(target_os = "windows")]
    fn get_version() -> Option<Self> {
        let windows_version = windows_version::OsVersion::current();

        let version = Version::from(&[windows_version.major, windows_version.minor, windows_version.pack]);

        debug!(
            "Retrieved current windows version version {version} with build version {}",
            windows_version.build
        );

        Some(Self::Windows { version })
    }

    /// Returns None for any unsupported OS.
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn get_version() -> Option<Self> {
        None
    }

    /// Gets the current OS type.
    pub fn get_os(&self) -> Os {
        match self {
            Self::MacOs { .. } => Os::MacOs,
            Self::Linux { .. } => Os::Linux,
            Self::Windows { .. } => Os::Windows,
            Self::Unknown => Os::Unknown,
        }
    }
}
