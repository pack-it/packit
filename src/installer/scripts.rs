use std::{
    collections::HashMap,
    fs,
    path::{self, Path, PathBuf},
    process::{Command, Stdio},
};

use thiserror::Error;

use crate::{
    cli,
    config::Config,
    installer::types::Version,
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, manager::RepositoryManager},
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

    #[error("Cannot fetch script from repository: {0}")]
    FetchScriptError(#[from] RepositoryError),
}

/// Runs the given pre install script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_pre_script<P: AsRef<Path>>(
    path: &PathBuf,
    run_dir: P,
    config: &Config,
    package_install_path: &PathBuf,
    args: &HashMap<&str, &str>,
) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, run_dir, config, env_vars, args)
}

/// Runs the given build script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_build_script<P: AsRef<Path>>(
    path: &PathBuf,
    run_dir: P,
    config: &Config,
    package_install_path: &PathBuf,
    args: &HashMap<&str, &str>,
) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, run_dir, config, env_vars, args)
}

/// Runs the given post install script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_post_script(
    path: &PathBuf,
    package_install_path: &PathBuf,
    config: &Config,
    args: &HashMap<&str, &str>,
) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, &package_install_path, config, env_vars, args)
}

/// Runs the given test script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_test_script(
    path: &PathBuf,
    package_install_path: &PathBuf,
    config: &Config,
    args: &HashMap<&str, &str>,
) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, &package_install_path, config, env_vars, args)
}

/// Runs the script at the given path, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_script<P: AsRef<Path>>(
    path: &PathBuf,
    run_dir: P,
    config: &Config,
    env_vars: HashMap<&str, &str>,
    args: &HashMap<&str, &str>,
) -> Result<(), ScriptError> {
    let path = to_absolute_path(path)?;

    let mut command = create_command(path);
    command
        .current_dir(run_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("PACKIT_PREFIX_PATH", &config.prefix_directory)
        .env("PACKIT_TARGET", TARGET_ARCHITECTURE);

    for (key, value) in env_vars {
        command.env(key, value);
    }

    // Add script arguments
    for (key, value) in args {
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
            cli::display_warning!("Script executed with status code {code}");
            Err(ScriptError::ScriptFailed(code))
        },
        None => {
            cli::display_warning!("Script executed without a status code");
            Err(ScriptError::ScriptFailed(-1))
        },
    }
}

/// Downloads a script and saves it as a temp file.
pub fn download_script(
    config: &Config,
    repository_manager: &RepositoryManager,
    script_name: &str,
    script_path: &str,
    package_name: &str,
    version: &Version,
    repository_id: &str,
) -> Result<Option<PathBuf>, ScriptError> {
    let name = format!("{package_name}_{version}_{script_name}");
    let script_destination = config.temp_directory.join(name).with_extension(SCRIPT_EXTENSION);

    match repository_manager.read_script(&repository_id, &package_name, &script_path)? {
        Some(script_text) => fs::write(&script_destination, &script_text).map_err(ScriptError::SaveError)?,
        None => return Ok(None), // Script not found, so return None
    }

    // Script succesfully downloaded, so return script location
    Ok(Some(script_destination))
}

fn to_absolute_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ScriptError> {
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
