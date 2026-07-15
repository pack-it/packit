// SPDX-License-Identifier: GPL-3.0-only
pub mod error;

#[cfg(target_os = "windows")]
mod windows;

use std::path::PathBuf;

use crate::platforms::tool_detection::error::Result;

#[cfg(target_os = "windows")]
pub use self::windows::detect_msvc;

#[cfg(not(target_os = "windows"))]
/// Detects if MSVC is installed on the system.
/// Returns the path to vcvarsall.bat if it is found, or `None` if MSVC is not found.
pub fn detect_msvc() -> Result<Option<PathBuf>> {
    Ok(None)
}
