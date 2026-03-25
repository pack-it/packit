/*
 * File containing the platform dependent defaults.
 */

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const DEFAULT_PREFIX: &str = "/opt/packit";

#[cfg(target_os = "windows")]
pub const DEFAULT_PREFIX: &str = "%PROGRAMFILES%/packit";

#[cfg(any(target_os = "linux"))]
pub const DEFAULT_CONFIG_DIR: &str = "/etc/packit";

#[cfg(any(target_os = "macos"))]
pub const DEFAULT_CONFIG_DIR: &str = "/Library/Application Support/packit";

#[cfg(target_os = "windows")]
pub const DEFAULT_CONFIG_DIR: &str = "%PROGRAMFILES%/packit";
