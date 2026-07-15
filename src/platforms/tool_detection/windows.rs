// SPDX-License-Identifier: GPL-3.0-only
use std::{path::PathBuf, process::Command};

use crate::{platforms::tool_detection::error::Result, utils::ioerror::IOResultExt};

/// Detects if MSVC is installed on the system.
/// Returns the path to vcvarsall.bat if it is found, or `None` if MSVC is not found.
pub fn detect_msvc() -> Result<Option<PathBuf>> {
    // Check if `vswhere` exists
    let vswhere = PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere");
    if !vswhere.exists() {
        return Ok(None);
    }

    // Read Visual Studio install path from `vswhere`
    let mut command = Command::new(vswhere);
    command.args(["-latest", "-property installationPath"]);
    let output = command.output().err_operation("run vswhere command")?;
    let path = str::from_utf8(&output.stdout)?;
    let path = PathBuf::from(path);

    // Check if install path exists
    if !path.exists() {
        return Ok(None);
    }

    // Check if vcvarsall.bat exists
    let vcvarsall = path.join("VC").join("Auxiliary").join("Build").join("vcvarsall.bat");
    if !vcvarsall.exists() {
        return Ok(None);
    }

    Ok(Some(vcvarsall))
}
