#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const DEFAULT_PREFIX: &str = "/opt/packit";

#[cfg(target_os = "windows")]
pub const DEFAULT_PREFIX: &str = "%APPDATA%/packit";

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const DEFAULT_CONFIG_DIR: &str = "/etc/packit";

#[cfg(target_os = "windows")]
pub const DEFAULT_CONFIG_DIR: &str = "%APPDATA%/packit";
