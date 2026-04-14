// SPDX-License-Identifier: GPL-3.0-only
pub mod error;

// Use correct platform implementation depending on the target os
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub mod unix;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use self::unix as platform;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use self::windows as platform;

use crate::platforms::permissions::error::Result;
use std::{fs, path::PathBuf};

pub const PACKIT_GROUP_NAME: &str = "packit";

/// Checks if a path is writable by the current user. Returns true if it is, false otherwise.
pub fn is_writable(path: &PathBuf) -> Result<bool> {
    if !fs::exists(path)? {
        return Ok(false);
    }

    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();

    // Check if path is read only
    if permissions.readonly() {
        return Ok(false);
    }

    // Use platform specific writable checks
    platform::is_writable(path, metadata)
}

/// Sets the permissions of packit files
pub use self::platform::set_packit_permissions;

pub use self::platform::PlatformError;

/// Platform implementation for unsupported targets
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use thiserror::Error;

    use super::error::Result;

    #[derive(Error, Debug)]
    pub enum PlatformError {}

    pub(super) fn is_writable(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        panic!("Cannot check write permissions for target, target is not supported.");
    }

    pub fn set_packit_permissions(_path: &PathBuf, _is_multiuser: bool, _recurse: bool) -> Result<()> {
        panic!("Cannot set ownership for target, target is not supported.");
    }
}
