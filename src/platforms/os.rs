use std::{str::FromStr, sync::LazyLock};

use crate::{
    cli::display::logging::{debug, error},
    installer::types::Version,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Os {
    MacOs,
    Linux,
    Windows,
    Unknown,
}

impl Os {
    pub fn is_unix(&self) -> bool {
        match self {
            Self::MacOs | Self::Linux => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
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

    #[cfg(target_os = "linux")]
    fn get_version() -> Option<Self> {
        use std::{fs, process::Command};

        let kernel_output = match Command::new("/usr/bin/uname").arg("-r").output() {
            Ok(output) => output,
            Err(e) => {
                error!(e, "Cannot fetch kernel version");
                return None;
            },
        };

        let kernel_version_str = match String::from_utf8(kernel_output.stdout) {
            Ok(string) => string,
            Err(e) => {
                error!(e, "Cannot parse kernel version to string");
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

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn get_version() -> Option<Self> {
        None
    }

    pub fn get_os(&self) -> Os {
        match self {
            Self::MacOs { .. } => Os::MacOs,
            Self::Linux { .. } => Os::Linux,
            Self::Windows { .. } => Os::Windows,
            Self::Unknown => Os::Unknown,
        }
    }
}
