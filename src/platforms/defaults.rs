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
/// Otherwise the `PACKIT_CONFIG_DIR` environment variable is saved in a `OnceLock` and used as the config directory path.
pub fn get_default_config_dir() -> &'static str {
    #[cfg(not(feature = "integration-tests"))]
    return DEFAULT_CONFIG_DIR;

    // Note that we use `OnceLock`, because the `DEFAULT_CONFIG_DIR` is a `const &str`, whereas `env::var` returns a `String`
    #[cfg(feature = "integration-tests")]
    {
        use std::sync::OnceLock;

        static CONFIG_DIR: OnceLock<String> = OnceLock::new();

        CONFIG_DIR
            .get_or_init(|| std::env::var("PACKIT_CONFIG_DIR").expect("No config directory specified by the integration test"))
            .as_str()
    }
}
