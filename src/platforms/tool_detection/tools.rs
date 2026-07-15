// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

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
}
