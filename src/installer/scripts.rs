use std::{
    collections::HashMap,
    fs,
    path::{self, Path, PathBuf},
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;
use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::{
        build_env::BuildEnv,
        types::{PackageName, Version},
    },
    platforms::TargetArchitecture,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::env::Environment,
};

/// The errors that occur during script handling.
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Cannot run script: {0}")]
    RunError(std::io::Error),

    #[error("Cannot transform path to absolute path: {0}")]
    AbsolutePathError(std::io::Error),

    #[error("Cannot save script to file: {0}")]
    SaveError(std::io::Error),

    #[error("Cannot parse PathBuf to string")]
    InvalidPathString,

    #[error("Cannot find script '{0}'")]
    ScriptNotFound(String),

    #[error("Script executed with status code {0}")]
    ScriptFailed(i32),

    #[error("Cannot fetch script from repository")]
    FetchScriptError(#[from] RepositoryError),
}

pub type Result<T> = core::result::Result<T, ScriptError>;

pub struct ScriptData<'a> {
    path: &'a dyn AsRef<Path>,
    package_install_path: &'a dyn AsRef<Path>,
    package_version: &'a Version,
    config: &'a Config,
    args: &'a HashMap<&'a str, &'a str>,
}

impl<'a> ScriptData<'a> {
    pub fn new(
        path: &'a impl AsRef<Path>,
        package_install_path: &'a impl AsRef<Path>,
        package_version: &'a Version,
        config: &'a Config,
        args: &'a HashMap<&str, &str>,
    ) -> Self {
        Self {
            path,
            package_install_path,
            package_version,
            config,
            args,
        }
    }
}

/// Runs the given pre install script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_pre_script(script_data: &ScriptData, run_dir: impl AsRef<Path>) -> Result<()> {
    run_script(script_data, run_dir, Environment::new())
}

/// Runs the given build script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_build_script(script_data: &ScriptData, run_dir: impl AsRef<Path>, build_env: BuildEnv) -> Result<()> {
    run_script(script_data, run_dir, build_env.into())
}

/// Runs the given post install script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_post_script(script_data: &ScriptData) -> Result<()> {
    run_script(script_data, &script_data.package_install_path, Environment::new())
}

/// Runs the given test script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_test_script(script_data: &ScriptData) -> Result<()> {
    run_script(script_data, script_data.package_install_path, Environment::new())
}

/// Runs the given uninstall script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_uninstall_script(script_data: &ScriptData) -> Result<()> {
    run_script(script_data, script_data.package_install_path, Environment::new())
}

/// Runs the script at the given path, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
fn run_script(script_data: &ScriptData, run_dir: impl AsRef<Path>, env: Environment) -> Result<()> {
    let path = to_absolute_path(&script_data.path)?;

    let package_install_path = to_absolute_path(script_data.package_install_path)?;
    let package_install_path = package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?;

    let mut command = create_command(path);
    command
        .current_dir(run_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("PACKIT_PREFIX_PATH", &script_data.config.prefix_directory)
        .env("PACKIT_TARGET", TargetArchitecture::current().to_string())
        .env("PACKIT_PACKAGE_PATH", package_install_path)
        .env("PACKIT_PACKAGE_VERSION", script_data.package_version.to_string());

    // Remove stripped environment variables
    for key in env.stripped_vars {
        command.env_remove(key);
    }

    // Add script environment variables
    for (key, value) in env.env_vars {
        command.env(key, value);
    }

    // Add script arguments
    for (key, value) in script_data.args {
        let formatted_key = format!("PACKIT_ARGS_{}", key.to_uppercase());
        command.env(formatted_key, value);
    }

    // Run script
    let output = command.output().map_err(ScriptError::RunError)?;

    // Display status to user
    match output.status.code() {
        Some(0) => {
            println!("Script executed succesfully.");
            Ok(())
        },
        Some(code) => {
            warning!("Script executed with status code {code}");
            Err(ScriptError::ScriptFailed(code))
        },
        None => {
            warning!("Script executed without a status code");
            Err(ScriptError::ScriptFailed(-1))
        },
    }
}

/// Downloads a script and saves it as a temp file.
pub fn download_script(
    repository_manager: &RepositoryManager,
    script_path: &str,
    package_name: &PackageName,
    repository_id: &str,
) -> Result<Option<NamedTempFile>> {
    let script_text = match repository_manager.read_script(&repository_id, &package_name, &script_path)? {
        Some(script_text) => script_text,
        None => return Ok(None), // Script not found, so return None
    };

    // Write script to file
    Ok(Some(write_script_to_tempfile(&script_text)?))
}

/// Downloads a script and saves it as a temp file.
pub fn write_script_to_tempfile(script_text: &str) -> Result<NamedTempFile> {
    // Write script to file
    let file = NamedTempFile::new().map_err(ScriptError::SaveError)?;
    fs::write(&file, &script_text).map_err(ScriptError::SaveError)?;

    // Return created tempfile
    Ok(file)
}

fn to_absolute_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    Ok(path::absolute(path).map_err(|e| ScriptError::AbsolutePathError(e))?)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn create_command<P: AsRef<Path>>(path: P) -> Command {
    let mut command = Command::new("sh");
    command.arg("-eu").arg(path.as_ref());

    command
}

#[cfg(target_os = "windows")]
fn create_command<P: AsRef<Path>>(path: P) -> Command {
    let mut command = Command::new("cmd");
    command.arg("/C").arg(path.as_ref());

    command
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn create_command<P: AsRef<Path>>(path: P) -> Command {
    panic!("Cannot create command for target, target is not supported.");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const SCRIPT_EXTENSION: &'static str = "sh";

#[cfg(target_os = "windows")]
pub const SCRIPT_EXTENSION: &'static str = "bat";
