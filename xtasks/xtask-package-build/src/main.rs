// SPDX-License-Identifier: GPL-3.0-only
mod compress;
mod target_architecture;

use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;

use crate::target_architecture::TARGET_ARCHITECTURE;

/// Represents the package-build command, holding the given options.
#[derive(Parser, Debug)]
#[command(name = "package-build", version, about)]
pub struct TaskCommand {
    /// The directory to save the package to
    #[arg(long)]
    destination: Option<PathBuf>,

    /// True if the package should be overwritten if it already exists
    #[arg(short, long, default_value = "false")]
    overwrite: bool,
}

const PACKIT_VERSION: &str = "0.0.2";
const PACKIT_REVISION: &str = "0";

fn main() {
    let command = TaskCommand::parse();

    // Check if build exists
    let source = get_target_path().join("build");
    if !source.exists() {
        eprintln!("Failed to find Packit build, please make sure to run cargo build-install before packaging");
        exit(1);
    }

    // Retrieve destination
    let destination = command.destination.unwrap_or(get_target_path());
    let destination = match std::path::absolute(destination) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to convert destination directory to absolute path");
            eprintln!("Error: {e}");
            exit(1);
        },
    };

    // Check if package already exists
    let package_name = format!("packit@{PACKIT_VERSION}-{PACKIT_REVISION}-{TARGET_ARCHITECTURE}");
    let package_file_name = format!("{package_name}.tar.gz");
    let archive_destination = destination.join(&package_file_name);
    if !command.overwrite && archive_destination.exists() {
        eprintln!("The package already exists, please remove it first or specifiy the -o option");
        exit(1);
    }

    // Compress build
    let bytes = match compress::compress(&source, &package_name) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to compress build");
            eprintln!("Error: {e}");
            exit(1);
        },
    };

    // Write archive to destination
    if let Err(e) = fs::write(archive_destination, bytes) {
        eprintln!("Failed to write compressed build to destination");
        eprintln!("Error: {e}");
        exit(1);
    }

    println!("Finished packaging Packit to {}", destination.display())
}

/// Gets the path to the project root directory.
fn get_project_root_path() -> PathBuf {
    Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("Expected CARGO_MANIFEST_DIR to return the xtask-package-build directory path.")
        .to_path_buf()
}

/// Gets the target directory path in the project root.
fn get_target_path() -> PathBuf {
    get_project_root_path().join("target")
}
