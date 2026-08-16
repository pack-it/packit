// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, path::PathBuf};

use crate::{cli::display::logging::warning, integrity::error::Result, utils::ioerror::IOResultExt};

/// Fixes stray directories by removing them.
pub fn fix_stray_directories(strays: HashSet<PathBuf>) -> Result<()> {
    for directory in strays {
        if !fs::exists(&directory).err_with_path("check existence of", &directory)? {
            warning!(
                "Skipping deletion of stray directory '{}' because it doesn't exist.",
                directory.display()
            );
        }

        match directory.is_dir() {
            true => fs::remove_dir_all(&directory).err_with_path("remove dirs", directory)?,
            false => fs::remove_file(&directory).err_with_path("remove file", directory)?,
        }
    }

    Ok(())
}

/// Removes all the invalid files. Note that symlinks aren't (and shouldn't be) traversed.
pub fn fix_invalid_files(invalid: &Vec<PathBuf>) -> Result<()> {
    for file in invalid {
        if file.is_dir() {
            fs::remove_dir_all(file).err_with_path("remove dirs", file)?;
            continue;
        }

        fs::remove_file(file).err_with_path("remove file", file)?;
    }

    Ok(())
}
