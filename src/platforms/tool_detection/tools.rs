// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use crate::platforms::Target;

/// Represents the Microsoft Visual C++ toolchain.
pub struct Msvc {
    vs_path: PathBuf,
}

impl Msvc {
    /// Creates a new `Msvc`, holding the path to the toolchain.
    #[cfg(windows)]
    pub fn new(vs_path: PathBuf) -> Self {
        Self { vs_path }
    }

    /// Gets the installation path of Visual Studio.
    pub fn get_vs_path(&self) -> &PathBuf {
        &self.vs_path
    }

    /// Gets the path of the vcvarsall.bat script.
    pub fn get_vcvarsall_path(&self) -> PathBuf {
        self.vs_path.join("VC").join("Auxiliary").join("Build").join("vcvarsall.bat")
    }

    /// Gets the target needed for the vcvarsall.bat execution.
    /// Returns `None` if the given target is not supported
    pub fn get_vcvarsall_target(&self, target: &Target) -> Option<&str> {
        match target.architecture {
            crate::platforms::TargetArchitecture::WindowsX86_64Msvc => Some("x64"),
            crate::platforms::TargetArchitecture::WindowsAarch64Msvc => Some("arm64"),
            _ => None,
        }
    }
}
