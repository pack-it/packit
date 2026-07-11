// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf};

use crate::{
    config::Config,
    integrity::{Issue, error::Result},
    platforms::{DEFAULT_PREFIX, permissions::is_writable},
    register::package_register::PackageRegister,
    utils::ioerror::IOResultExt,
};

/// Checks the permissions of the prefix directory and all its sub directories.
/// If the config can be used it will use the prefix directory specified there,
/// otherwise the default prefix directory is checked.
pub fn check_permissions() -> Result<Option<Issue>> {
    let prefix_directory = match Config::from(&Config::get_default_path()) {
        Ok(config) => config.prefix_directory,
        Err(_) => DEFAULT_PREFIX.into(),
    };

    let unwritable = check_permissions_impl(&prefix_directory)?;
    if unwritable.is_empty() {
        return Ok(None);
    }

    Ok(Some(Issue::IncorrectPermissions(unwritable.into_iter().collect())))
}

/// Recursively checks if all files in directory are writable.
/// Returns all directories which are not writable (could be empty).
fn check_permissions_impl(directory: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut unwritable = Vec::new();

    // Return early if the directory doesn't exist (also if a symlink destination doesn't exist)
    if !directory.exists() {
        return Ok(unwritable);
    }

    // Skip file if it has the read only flag enabled on Windows
    #[cfg(windows)]
    {
        let metadata = fs::metadata(directory).err_with_path("read metadata of", directory)?;
        if !directory.is_dir() && metadata.permissions().readonly() {
            return Ok(unwritable);
        }
    }

    if !is_writable(directory)? {
        unwritable.push(directory.clone());
    }

    if !directory.is_dir() {
        return Ok(unwritable);
    }

    // Recurse
    for sub_directory in fs::read_dir(directory).err_with_path("read", directory)? {
        let sub_directory = sub_directory.err_with_path("iterate", directory)?;
        unwritable.extend(check_permissions_impl(&sub_directory.path())?);
    }

    Ok(unwritable)
}

/// Checks if the Config.toml exists.
/// Returns `None` if the config exists or an `Issue::MissingConfig` otherwise.
/// Could return an IO error.
pub fn check_config_existence() -> Result<Option<Issue>> {
    if fs::exists(Config::get_default_path()).err_with_path("check existance of", Config::get_default_path())? {
        return Ok(None);
    }

    Ok(Some(Issue::MissingConfig))
}

/// Checks if the `Config.toml` syntax is valid.
/// Returns `None` if the config syntax is valid or an `Issue::MissingConfig` otherwise.
/// Could return an IO error.
pub fn check_config_syntax() -> Result<Option<Issue>> {
    // Don't return a config syntax issue if the `Config.toml` doesn't exist
    if !fs::exists(Config::get_default_path()).err_with_path("check existance of", Config::get_default_path())? {
        return Ok(None);
    }

    match Config::from(&Config::get_default_path()) {
        Ok(_) => Ok(None),
        Err(_) => Ok(Some(Issue::MissingConfig)),
    }
}

/// Checks if the Register.toml exists.
/// Returns `None` if the register exists or an `Issue::MissingRegister` otherwise.
pub fn check_register_existence() -> Result<Option<Issue>> {
    let config = Config::from(&Config::get_default_path())?;
    let register_directory = &PackageRegister::get_path(&config.prefix_directory);
    if fs::exists(register_directory).err_with_path("check existance of", register_directory)? {
        return Ok(None);
    }

    Ok(Some(Issue::MissingRegister))
}

/// Checks if the Register.toml syntax is valid.
/// Returns `None` if the register syntax is valid or an `Issue::MissingRegister` otherwise.
pub fn check_register_syntax() -> Result<Option<Issue>> {
    let config = Config::from(&Config::get_default_path())?;
    let register_directory = &PackageRegister::get_path(&config.prefix_directory);

    // Don't return a register syntax issue if the `Register.toml` doesn't exist
    if !fs::exists(register_directory).err_with_path("check existance of", register_directory)? {
        return Ok(None);
    }

    match PackageRegister::from(register_directory) {
        Ok(_) => Ok(None),
        Err(_) => Ok(Some(Issue::MissingRegister)),
    }
}
