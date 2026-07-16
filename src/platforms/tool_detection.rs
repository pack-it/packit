// SPDX-License-Identifier: GPL-3.0-only
pub mod error;
pub mod tools;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
use crate::platforms::tool_detection::{error::Result, tools::Msvc};

#[cfg(target_os = "windows")]
pub use self::windows::detect_msvc;

/// Detects if MSVC is installed on the system.
/// Returns the installation path of Visual Studio if it is found, or `None` if MSVC is not found.
#[cfg(not(target_os = "windows"))]
pub fn detect_msvc() -> Result<Option<Msvc>> {
    Ok(None)
}
