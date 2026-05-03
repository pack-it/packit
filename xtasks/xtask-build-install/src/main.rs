use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, exit},
};

use clap::Parser;

/// Represents the build-install command, holding the given options.
#[derive(Parser, Debug)]
#[command(name = "build-install", version, about)]
pub struct TaskCommand {
    /// The destination to install to
    #[arg(long)]
    destination: Option<PathBuf>,
}

fn main() {
    let command = TaskCommand::parse();

    // Build with cargo
    run_build();

    // Retrieve destination
    let destination = command.destination.unwrap_or(get_target_path().join("build"));
    let destination = match std::path::absolute(destination) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to convert destination directory to absolute path");
            eprintln!("Error: {e}");
            exit(1);
        },
    };

    // Create destination
    if let Err(e) = fs::create_dir_all(&destination) {
        eprintln!("Failed to create destination directory");
        eprintln!("Error: {e}");
        exit(1);
    }

    // Copy files to destination
    copy_binary_to_destination(&destination);

    println!("Finished building and installing Packit to {}", destination.display())
}

/// Runs the Packit build using `cargo build --release`.
fn run_build() {
    // Create build command
    let mut cmd = Command::new(std::env!("CARGO"));
    cmd.args(["build", "--release"]);

    // Run build command and handle output
    match cmd.status() {
        Ok(status) if !status.success() => {
            eprintln!("Failed to build Packit, failed command: `{cmd:?}`");
            exit(status.code().unwrap_or(1));
        },
        Ok(_) => {
            println!("Packit build successful");
        },
        Err(e) => {
            eprintln!("Failed to build Packit, failed command: `{cmd:?}`");
            eprintln!("Error: {e}");
            exit(1);
        },
    }
}

/// Copies the `packit` binary to the destinationa and creates the `pit` symlink.
fn copy_binary_to_destination(destination: &PathBuf) {
    // Create destination bin directory
    let bin_directory = destination.join("bin");
    if let Err(e) = fs::create_dir_all(&bin_directory) {
        eprintln!("Failed to create bin directory");
        eprintln!("Error: {e}");
        exit(1);
    }

    let binary_path = get_target_path().join("release").join("packit");

    // Copy packit binary to bin directory
    if let Err(e) = fs::copy(binary_path, bin_directory.join("packit")) {
        eprintln!("Failed to copy binary to bin directory");
        eprintln!("Error: {e}");
        exit(1);
    }

    // Create symlink for pit to packit
    if let Err(e) = create_file_symlink("packit", bin_directory.join("pit")) {
        eprintln!("Failed to create pit symlink in bin directory");
        eprintln!("Error: {e}");
        exit(1);
    }
}

/// Gets the path to the proejct root directory.
fn get_project_root_path() -> PathBuf {
    Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("Expected CARGO_MANIFEST_DIR to return the xtask-build-install directory path.")
        .to_path_buf()
}

/// Gets the target directory path in the project root.
fn get_target_path() -> PathBuf {
    get_project_root_path().join("target")
}

/// Creates a file symlink. The link at `link` will point to `original`.
fn create_file_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> std::io::Result<()> {
    #[cfg(unix)]
    return std::os::unix::fs::symlink(original, link);

    #[cfg(windows)]
    return std::os::windows::fs::symlink_file(original, link);

    #[cfg(not(any(windows, unix)))]
    {
        eprintln!("Failed to create symlink: Platform not supported");
        exit(1);
    }
}
