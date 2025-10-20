#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCHITECTURE: &str = "x86_64-apple-darwin";

#[cfg(target_os = "macos")]
#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCHITECTURE: &str = "aarch64-apple-darwin";

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
#[cfg(target_env = "gnu")]
pub const TARGET_ARCHITECTURE: &str = "unknown-linux-gnu";
