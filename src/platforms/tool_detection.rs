// SPDX-License-Identifier: GPL-3.0-only
pub mod error;
pub mod tools;

#[cfg(target_os = "windows")]
mod windows;

use crate::platforms::tool_detection::{error::Result, tools::Msvc};

#[cfg(target_os = "windows")]
pub use self::windows::detect_msvc;

#[cfg(not(target_os = "windows"))]
/// Detects if MSVC is installed on the system.
/// Returns the path to vcvarsall.bat if it is found, or `None` if MSVC is not found.
pub fn detect_msvc() -> Result<Option<Msvc>> {
    Ok(None)
}
