// SPDX-License-Identifier: GPL-3.0-only
/*
 * A constants file to keep global constants.
 */
use crate::repositories::metadata::WEB_METADATA_PROVIDER_ID;

pub const REGISTER_FILENAME: &str = "Register.toml";

/// Note that when the `test-support` feature flag is enabled `CONFIG_FILENAME` is compiled differently.
#[cfg(feature = "test-support")]
pub const CONFIG_FILENAME: &str = "TestConfig.toml";

#[cfg(not(feature = "test-support"))]
pub const CONFIG_FILENAME: &str = "Config.toml";

pub const DEFAULT_METADATA_REPOSITORY_NAME: &str = "core";
pub const DEFAULT_METADATA_REPOSITORY_URL: &str = "https://raw.githubusercontent.com/pack-it/core/main/";
pub const DEFAULT_METADATA_REPOSITORY_PROVIDER: &str = WEB_METADATA_PROVIDER_ID;
