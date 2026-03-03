use std::{fmt::Display, str::FromStr};

use crate::platforms::Os;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetArchitecture {
    MacOsX86_64,
    MacOsAarch64,
    LinuxAarch64Gnu,
    LinuxX86_64Gnu,
    LinuxX86_64Musl,
    WindowsX86_64Msvc,
    WindowsAarch64Msvc,
    Unknown(Option<String>),
}

impl Display for TargetArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::MacOsX86_64 => "x86_64-apple-darwin",
            Self::MacOsAarch64 => "aarch64-apple-darwin",
            Self::LinuxAarch64Gnu => "aarch64-unknown-linux-gnu",
            Self::LinuxX86_64Gnu => "x86_64-unknown-linux-gnu",
            Self::LinuxX86_64Musl => "x86_64-unknown-linux-musl",
            Self::WindowsX86_64Msvc => "x86_64-pc-windows-msvc",
            Self::WindowsAarch64Msvc => "aarch64-pc-windows-msvc",
            Self::Unknown(Some(string)) => string,
            Self::Unknown(None) => "unknown",
        };

        write!(f, "{string}")
    }
}

impl FromStr for TargetArchitecture {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "x86_64-apple-darwin" => Ok(Self::MacOsX86_64),
            "aarch64-apple-darwin" => Ok(Self::MacOsAarch64),
            "aarch64-unknown-linux-gnu" => Ok(Self::LinuxAarch64Gnu),
            "x86_64-unknown-linux-gnu" => Ok(Self::LinuxX86_64Gnu),
            "x86_64-unknown-linux-musl" => Ok(Self::LinuxX86_64Musl),
            "x86_64-pc-windows-msvc" => Ok(Self::WindowsX86_64Msvc),
            "aarch64-pc-windows-msvc" => Ok(Self::WindowsAarch64Msvc),
            _ => Err(()), // Do we want to error or return unknown here?
        }
    }
}

impl TargetArchitecture {
    pub fn current() -> Self {
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Self::MacOsX86_64;

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Self::MacOsAarch64;

        #[cfg(all(target_os = "linux", target_arch = "aarch64", target_env = "gnu"))]
        return Self::LinuxAarch64Gnu;

        #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
        return Self::LinuxX86_64Gnu;

        #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
        return Self::LinuxX86_64Musl;

        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
        return Self::WindowsX86_64Msvc;

        #[cfg(all(target_os = "windows", target_arch = "aarch64", target_env = "msvc"))]
        return Self::WindowsAarch64Msvc;

        #[cfg(not(any(
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "aarch64", target_env = "gnu"),
            all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"),
            all(target_os = "linux", target_arch = "x86_64", target_env = "musl"),
            all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"),
            all(target_os = "windows", target_arch = "aarch64", target_env = "msvc"),
        )))]
        Self::Unknown(None)
    }

    pub fn get_os(&self) -> Os {
        match self {
            Self::MacOsX86_64 | Self::MacOsAarch64 => Os::MacOs,
            Self::LinuxAarch64Gnu | Self::LinuxX86_64Gnu | Self::LinuxX86_64Musl => Os::Linux,
            Self::WindowsX86_64Msvc | Self::WindowsAarch64Msvc => Os::Windows,
            Self::Unknown(_) => Os::Unknown,
        }
    }
}
