// SPDX-License-Identifier: GPL-3.0-only
use std::{path::PathBuf, process::Command};

use crate::{
    cli::display::logging::debug,
    platforms::tool_detection::{error::Result, tools::Msvc},
    utils::ioerror::IOResultExt,
};

/// Detects if MSVC is installed on the system.
/// Returns the installation path of Visual Studio if it is found, or `None` if MSVC is not found.
pub fn detect_msvc() -> Result<Option<Msvc>> {
    // Check if `vswhere` exists
    let vswhere = PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe");
    if !vswhere.exists() {
        debug!("Cannot find 'vswhere.exe'");
        return Ok(None);
    }

    // Read Visual Studio install path from `vswhere`
    let mut command = Command::new(vswhere);
    command.args(["-latest", "-property", "installationPath"]);
    let output = command.output().err_operation("run vswhere command")?;
    let vs_path = str::from_utf8(&output.stdout)?;
    let vs_path = PathBuf::from(vs_path.trim());

    // Check if install path exists
    if !vs_path.exists() {
        debug!("The Visual Studio install does not exist at '{}'", vs_path.display());
        return Ok(None);
    }

    let msvc = Msvc::new(vs_path);

    // Check if vcvarsall.bat exists
    let vcvarsall = msvc.get_vcvarsall_path();
    if !vcvarsall.exists() {
        debug!("The vcvarsall.bat script does not exist at '{}'", vcvarsall.display());
        return Ok(None);
    }

    Ok(Some(msvc))
}
