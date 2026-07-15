// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use crate::{
    installer::types::Version,
    platforms::{Target, TargetArchitecture},
};

/// Represents the Microsoft Visual C++ toolchain.
pub struct Msvc {
    vs_path: PathBuf,
    version: Version,
}

impl Msvc {
    /// Creates a new `Msvc`, holding the path to the toolchain.
    #[cfg(windows)]
    pub fn new(vs_path: PathBuf, version: Version) -> Self {
        Self { vs_path, version }
    }

    /// Gets the installation path of Visual Studio.
    pub fn get_vs_path(&self) -> &PathBuf {
        &self.vs_path
    }

    /// Gets the version of the MSVC toolchain.
    pub fn get_version(&self) -> &Version {
        &self.version
    }

    /// Gets the path of the vcvarsall.bat script.
    pub fn get_vcvarsall_path(&self) -> PathBuf {
        self.vs_path.join("VC").join("Auxiliary").join("Build").join("vcvarsall.bat")
    }

    /// Gets the arch needed for the vcvarsall.bat execution.
    /// Returns `None` if the given target is not supported
    pub fn get_vcvarsall_arch(&self, target: &Target) -> Option<&str> {
        match target.architecture {
            TargetArchitecture::WindowsX86_64Msvc => Some("x64"),
            TargetArchitecture::WindowsAarch64Msvc => Some("arm64"),
            _ => None,
        }
    }
}
