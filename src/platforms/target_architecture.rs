#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCHITECTURE: &str = "x86_64-apple-darwin";

#[cfg(target_os = "macos")]
#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCHITECTURE: &str = "aarch64-apple-darwin";

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
#[cfg(target_env = "gnu")]
pub const TARGET_ARCHITECTURE: &str = "aarch64-unknown-linux-gnu";

#[cfg(target_os = "linux")]
#[cfg(target_arch = "x86_64")]
#[cfg(target_env = "gnu")]
pub const TARGET_ARCHITECTURE: &str = "x86_64-unknown-linux-gnu";

#[cfg(target_os = "linux")]
#[cfg(target_arch = "x86_64")]
#[cfg(target_env = "musl")]
pub const TARGET_ARCHITECTURE: &str = "x86_64-unknown-linux-musl";

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
#[cfg(target_env = "msvc")]
pub const TARGET_ARCHITECTURE: &str = "x86_64-pc-windows-msvc";

#[cfg(target_os = "windows")]
#[cfg(target_arch = "aarch64")]
#[cfg(target_env = "msvc")]
pub const TARGET_ARCHITECTURE: &str = "aarch64-pc-windows-msvc";

pub fn get_os_name(target_name: &str) -> &str {
    match target_name {
        "x86_64-apple-darwin" | "aarch64-apple-darwin" => "mac",
        "aarch64-unknown-linux-gnu" | "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "linux",
        "x86_64-pc-windows-msvc" | "aarch64-pc-windows-msvc" => "windows",
        _ => unreachable!(), //TODO: refactor target architecture to be an enum
    }
}

pub fn is_unix(target_name: &str) -> bool {
    match target_name {
        "x86_64-apple-darwin"
        | "aarch64-apple-darwin"
        | "aarch64-unknown-linux-gnu"
        | "x86_64-unknown-linux-gnu"
        | "x86_64-unknown-linux-musl" => true,
        _ => false,
    }
}
