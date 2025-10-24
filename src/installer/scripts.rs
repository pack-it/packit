use std::{
    io::{self, Write},
    path::Path,
    process::Command,
};

use crate::target_architecture::TARGET_ARCHITECTURE;

/// Runs the script at the given path, in the given directory
/// Note that the script should be a `.sh` script on Linux and macOS and a `.bat` on Windows.
pub fn run_script<P: AsRef<Path>>(path: &str, run_dir: P) -> Result<(), io::Error> {
    let mut command = create_command(path);
    command
        .current_dir(run_dir)
        .env("PACKIT_PATH", "./temp") //TODO: Fetch from config
        .env("PACKIT_TARGET", TARGET_ARCHITECTURE);

    // Run script
    let output = command.output()?;

    // Write output to stdout
    io::stdout().write_all(&output.stdout)?;

    // Write warnings to stdout
    io::stdout().write_all(&output.stderr)?;

    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn create_command(path: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg(path);

    command
}

#[cfg(target_os = "windows")]
fn create_command(path: &str) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", path]);

    command
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn create_command(path: &str) -> Command {
    panic!("Cannot create command for target, target is not supported.");
}
