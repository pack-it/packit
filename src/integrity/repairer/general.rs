// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, fs, path::PathBuf};

use crate::{cli::display::logging::warning, integrity::error::Result};

/// Fixes stray directories by removing them.
pub fn fix_stray_directories(strays: HashSet<PathBuf>) -> Result<()> {
    for directory in strays {
        if !fs::exists(&directory)? {
            warning!(
                "Skipping deletion of stray directory '{}' because it doesn't exist.",
                directory.display()
            );
        }

        match directory.is_dir() {
            true => fs::remove_dir_all(directory)?,
            false => fs::remove_file(directory)?,
        }
    }

    Ok(())
}
