// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, VecDeque},
    fs::{self},
    io::{BufRead, BufReader},
    path::{self, Path, PathBuf},
    process::{Command, Stdio},
};

use bytes::Bytes;
use tempfile::{NamedTempFile, TempDir};
use thiserror::Error;

use crate::{
    builder::BuildEnv,
    cli::display::logging::warning,
    config::Config,
    installer::types::{PackageId, PackageName},
    platforms::{Os, TargetArchitecture},
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::{
        env::Environment,
        ioerror::{self, IOResultExt},
    },
};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::{fd::IntoRawFd, unix::process::CommandExt};

/// The errors that occur during script handling.
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Cannot parse PathBuf to string")]
    InvalidPathString,

    #[error("Cannot find script '{0}'")]
    ScriptNotFound(String),

    #[error("Script executed with status code {0}")]
    ScriptFailed(i32),

    #[error("Cannot run script")]
    RunError(#[source] std::io::Error),

    #[error("Cannot transform path to absolute path")]
    AbsolutePathError(#[source] std::io::Error),

    #[error("Cannot save script or test file to file")]
    SaveError(#[source] std::io::Error),

    #[error("Cannot create temp directory for script to run")]
    TempCreationError(#[source] std::io::Error),

    #[error("Cannot fetch script from repository")]
    FetchScriptError(#[from] RepositoryError),

    #[error("IOError during a script operation")]
    IOError(#[from] ioerror::IOError),
}

pub type Result<T> = core::result::Result<T, ScriptError>;

/// Describes the number of lines that should be captured from a script that has show_output disabled.
const MAX_CAPTURED_OUTPUT_LINES: usize = 10;

/// Holds data necessary for script execution.
pub struct ScriptData<'a> {
    path: &'a dyn AsRef<Path>,
    package_install_path: &'a dyn AsRef<Path>,
    package_id: &'a PackageId,
    config: &'a Config,
    args: &'a HashMap<&'a str, &'a str>,
    verbose: bool,
}

impl<'a> ScriptData<'a> {
    /// Creates a new `ScriptData` instance.
    pub fn new(
        path: &'a impl AsRef<Path>,
        package_install_path: &'a impl AsRef<Path>,
        package_id: &'a PackageId,
        config: &'a Config,
        args: &'a HashMap<&str, &str>,
        verbose: bool,
    ) -> Self {
        Self {
            path,
            package_install_path,
            package_id,
            config,
            args,
            verbose,
        }
    }
}

/// Runs the given pre install script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_pre_script(script_data: &ScriptData, run_dir: impl AsRef<Path>) -> Result<()> {
    run_script(script_data, run_dir, Environment::new(), true)
}

/// Runs the given build script, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_build_script(script_data: &ScriptData, run_dir: impl AsRef<Path>, build_env: BuildEnv) -> Result<()> {
    run_script(script_data, run_dir, build_env.into(), script_data.verbose)
}

/// Runs the given post install script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_post_script(script_data: &ScriptData) -> Result<()> {
    run_script(script_data, &script_data.package_install_path, Environment::new(), true)
}

/// Runs the given test script, in a newly created temp directory.
/// It also writes the specified external test files to the temp directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_test_script(script_data: &ScriptData, external_files: &Vec<(&String, Bytes)>) -> Result<()> {
    let temp_dir = TempDir::new().map_err(ScriptError::TempCreationError)?;

    // Install external files into the temp directory
    for (file_name, file_content) in external_files {
        fs::write(temp_dir.path().join(file_name), file_content).map_err(ScriptError::SaveError)?;
    }

    run_script(script_data, &temp_dir, Environment::new(), true)
}

/// Runs the given uninstall script, in the package install directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_uninstall_script(script_data: &ScriptData) -> Result<()> {
    run_script(script_data, script_data.package_install_path, Environment::new(), true)
}

/// Runs the script at the given path, in the given directory.
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
fn run_script(script_data: &ScriptData, run_dir: impl AsRef<Path>, env: Environment, show_output: bool) -> Result<()> {
    let path = to_absolute_path(script_data.path)?;

    let package_install_path = to_absolute_path(script_data.package_install_path)?;
    let package_install_path = package_install_path.to_str().ok_or(ScriptError::InvalidPathString)?;

    let package_dependencies_path = script_data.config.prefix_directory.join("dependencies").join(script_data.package_id.to_string());

    let mut command = create_command(path);
    command
        .current_dir(run_dir)
        .env("PACKIT_PREFIX_PATH", &script_data.config.prefix_directory)
        .env("PACKIT_TARGET", TargetArchitecture::current().to_string())
        .env("PACKIT_OS", Os::current().as_str())
        .env("PACKIT_PACKAGE_PATH", package_install_path)
        .env("PACKIT_PACKAGE_VERSION", script_data.package_id.version.to_string())
        .env("PACKIT_PACKAGE_DEPENDENCIES_PATH", &package_dependencies_path)
        .env("PACKIT_VERBOSE", if script_data.verbose { "1" } else { "0" });

    // Only display build logging if verbose is enabled, otherwise create combined pipe for reading
    let mut output = None;
    if show_output {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        // Create pipe to capture both stdout and stderr together
        let (reader, writer) = std::io::pipe().err_operation("create pipe")?;
        let writer_clone = writer.try_clone().err_operation("clone pipe writer")?;
        command.stdout(writer).stderr(writer_clone);
        output = Some(reader);
    }

    // Bind extra output pipes
    bind_extra_outputs(&mut command, script_data)?;

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

    // Run script and destroy command builder
    let mut process = command.spawn().map_err(ScriptError::RunError)?;
    drop(command);

    // Capture last lines of script output if enabled
    let mut last_lines = VecDeque::with_capacity(MAX_CAPTURED_OUTPUT_LINES);
    if let Some(output) = output {
        let lines = BufReader::new(output).lines();
        for line in lines {
            let line = line.err_operation("read output line")?;

            if last_lines.len() == MAX_CAPTURED_OUTPUT_LINES {
                last_lines.pop_front();
            }

            last_lines.push_back(line);
        }
    }

    // Display status to user
    let status = process.wait().map_err(ScriptError::RunError)?;
    match status.code() {
        Some(0) => {
            println!("Script executed succesfully");
            Ok(())
        },
        Some(code) => {
            warning!("Script executed with status code {code}");
            if !last_lines.is_empty() {
                eprintln!("Last lines of script output:");
                last_lines.iter().for_each(|x| eprintln!("{x}"));
            }

            Err(ScriptError::ScriptFailed(code))
        },
        None => {
            warning!("Script executed without a status code");
            if !last_lines.is_empty() {
                eprintln!("Last lines of script output:");
                last_lines.iter().for_each(|x| eprintln!("{x}"));
            }

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
    let script_text = match repository_manager.read_file(repository_id, package_name, script_path)? {
        Some(script_text) => script_text,
        None => return Ok(None), // Script not found, so return None
    };

    // Write script to file
    Ok(Some(write_script_to_tempfile(&script_text)?))
}

/// Downloads a script and saves it as a temp file.
pub fn write_script_to_tempfile(script_text: &str) -> Result<NamedTempFile> {
    let file = create_tempfile()?;

    // Write script to file
    fs::write(&file, script_text).map_err(ScriptError::SaveError)?;

    // Return created tempfile
    Ok(file)
}

#[cfg(not(target_os = "windows"))]
fn create_tempfile() -> Result<NamedTempFile> {
    NamedTempFile::new().map_err(ScriptError::SaveError)
}

#[cfg(target_os = "windows")]
fn create_tempfile() -> Result<NamedTempFile> {
    let mut builder = tempfile::Builder::new();
    builder.suffix(".bat");
    builder.tempfile().map_err(ScriptError::SaveError)
}

fn to_absolute_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    path::absolute(path).map_err(ScriptError::AbsolutePathError)
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
    command.arg("/Q").arg("/C").arg(path.as_ref());

    command
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn create_command<P: AsRef<Path>>(path: P) -> Command {
    panic!("Cannot create command for target, target is not supported.");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub const SCRIPT_EXTENSION: &str = "sh";

#[cfg(target_os = "windows")]
pub const SCRIPT_EXTENSION: &str = "bat";

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn bind_extra_outputs(command: &mut Command, script_data: &ScriptData) -> Result<()> {
    let source_verbose_fd = match script_data.verbose {
        true => libc::STDOUT_FILENO,
        false => fs::File::options().write(true).open("/dev/null").err_operation("open /dev/null")?.into_raw_fd(),
    };

    // SAFETY: only dup2 and close operations are called in the subprocess which are async-signal-safe.
    // SAFETY: ownership of the /dev/null file descriptor is moved into the subprocess and closed there.
    // SAFETY: errors of dup2 are propagated back to the spawn function.
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(source_verbose_fd, 3) == -1 {
                return Err(std::io::Error::last_os_error());
            }

            // Close /dev/null file descriptor after duplicating it onto fd 3
            if source_verbose_fd != libc::STDOUT_FILENO && source_verbose_fd != 3 {
                libc::close(source_verbose_fd);
            }

            Ok(())
        })
    };

    Ok(())
}

#[cfg(target_os = "windows")]
fn bind_extra_outputs(command: &mut Command, script_data: &ScriptData) -> Result<()> {
    if !script_data.verbose {
        command.env("PACKIT_OUTPUTS", "3>nul");
        return Ok(());
    }

    command.env("PACKIT_OUTPUTS", "3>&1");
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn bind_extra_outputs(command: &mut Command, script_data: &ScriptData) -> Result<()> {
    panic!("Cannot bind extra outputs for target, target is not supported.");
}
