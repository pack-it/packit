// SPDX-License-Identifier: GPL-3.0-only
/*
 * File containing the platform dependent defaults.
 */

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const DEFAULT_PREFIX: &str = "/opt/packit";

#[cfg(target_os = "windows")]
pub const DEFAULT_PREFIX: &str = "C:\\Program Files\\packit";

#[cfg(all(target_os = "linux", not(feature = "integration-tests")))]
const DEFAULT_CONFIG_DIR: &str = "/etc/packit";

#[cfg(all(target_os = "macos", not(feature = "integration-tests")))]
const DEFAULT_CONFIG_DIR: &str = "/Library/Application Support/packit";

#[cfg(all(target_os = "windows", not(feature = "integration-tests")))]
const DEFAULT_CONFIG_DIR: &str = "C:\\Program Files\\packit";

/// Returns the default config directory. It returns the constant defined above if the `integration-tests` feature is not enabled.
/// Otherwise the value from `TEST_CONFIG_PATH` (given by the test call) is used.
pub fn get_default_config_dir() -> &'static str {
    #[cfg(not(feature = "integration-tests"))]
    return DEFAULT_CONFIG_DIR;

    #[cfg(feature = "integration-tests")]
    {
        use crate::cli::commands::TEST_CONFIG_PATH;
        TEST_CONFIG_PATH.get().expect("Expected initialized config path")
    }
}
