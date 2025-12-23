use std::{
    collections::HashMap,
    fs,
    path::{self, Path, PathBuf},
    process::{Command, Stdio},
};

use thiserror::Error;

use crate::{cli, config::Config, platforms::TARGET_ARCHITECTURE};

/// The errors that occur during script handling.
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Cannot run script: {0}")]
    RunError(#[from] std::io::Error),

    #[error("Cannot transform path to absolute path: {0}")]
    AbsolutePathError(std::io::Error),

    #[error("Cannot save script to file: {0}")]
    SaveError(std::io::Error),

    #[error("Cannot parse PathBuf to string")]
    InvalidPathString,

    #[error("Cannot find script '{0}'")]
    ScriptNotFound(String),
}

/// Saves the given script text to the given destination.
pub fn save_script(script_text: &str, destination: &str) -> Result<(), ScriptError> {
    fs::write(destination, script_text).map_err(|e| ScriptError::SaveError(e))?;

    Ok(())
}

/// Runs the given pre install script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_pre_script<P: AsRef<Path>>(path: &str, run_dir: P, config: &Config, package_install_path: &str) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, run_dir, config, env_vars)
}

/// Runs the given build script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_build_script<P: AsRef<Path>>(path: &str, run_dir: P, config: &Config, package_install_path: &str) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, run_dir, config, env_vars)
}

/// Runs the given post install script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_post_script(path: &str, package_install_path: &str, config: &Config) -> Result<(), ScriptError> {
    let package_install_path = to_absolute_path(package_install_path)?;

    let env_vars = HashMap::from([(
        "PACKIT_PACKAGE_PATH",
        package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?,
    )]);

    run_script(path, &package_install_path, config, env_vars)
}

/// Runs the script at the given path, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_script<P: AsRef<Path>>(path: &str, run_dir: P, config: &Config, env_vars: HashMap<&str, &str>) -> Result<(), ScriptError> {
    let path = to_absolute_path(path)?;

    let mut command = create_command(path);
    command
        .current_dir(run_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("PACKIT_INSTALL_PATH", &config.install_directory)
        .env("PACKIT_TARGET", TARGET_ARCHITECTURE);

    for (key, value) in env_vars {
        command.env(key, value);
    }

    // Run script
    let output = command.output()?;

    // Display status to user
    match output.status.code() {
        Some(0) => println!("Script executed succesfully."),
        Some(code) => cli::display_warning(&format!("Script executed with status code {code}")),
        None => cli::display_warning("Script executed without a status code"),
    }

    Ok(())
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
