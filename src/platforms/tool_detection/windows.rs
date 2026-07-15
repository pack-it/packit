// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf, process::Command, str::FromStr};

use crate::{
    cli::display::logging::debug,
    installer::types::Version,
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

    // Read MSVC version
    let version_path = vs_path.join("VC").join("Auxiliary").join("Build").join("Microsoft.VCToolsVersion.default.txt");
    let version_str = fs::read_to_string(&version_path).err_with_path("read", version_path)?;
    let version = Version::from_str(&version_str)?;

    let msvc = Msvc::new(vs_path, version);

    // Check if vcvarsall.bat exists
    let vcvarsall = msvc.get_vcvarsall_path();
    if !vcvarsall.exists() {
        debug!("The vcvarsall.bat script does not exist at '{}'", vcvarsall.display());
        return Ok(None);
    }

    Ok(Some(msvc))
}
