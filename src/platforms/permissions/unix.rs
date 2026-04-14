// SPDX-License-Identifier: GPL-3.0-only
use std::{
    ffi::CString,
    fs::{self, Metadata, Permissions},
    os::unix::{
        self,
        fs::{MetadataExt, PermissionsExt},
    },
    path::PathBuf,
};

use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    platforms::permissions::{
        PACKIT_GROUP_NAME,
        error::{PermissionError, Result},
    },
};

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("String contains a nul byte")]
    NulError(#[from] std::ffi::NulError),
}

/// Checks if a directory is writeable. Returns true if it is, false otherwise.
pub(super) fn is_writable(_path: &PathBuf, metadata: Metadata) -> Result<bool> {
    let mode = metadata.mode();

    // Check if path is writable for everyone
    if mode & 0o002 != 0 {
        return Ok(true);
    }

    // Check if path is writable for current group
    let current_group = unsafe { libc::getegid() };
    let group = metadata.gid();
    if current_group == group && mode & 0o020 != 0 {
        return Ok(true);
    }

    // Check if path is writable for current user
    let current_user = unsafe { libc::geteuid() };
    let user = metadata.uid();
    if current_user == user && mode & 0o200 != 0 {
        return Ok(true);
    }

    Ok(false)
}

/// Sets the permissions of packit files.
/// Could return a `PermissionError::GroupDoesNotExist` if the packit group does not exist when using multiuser mode or an IO error.
pub fn set_packit_permissions(path: &PathBuf, is_multiuser: bool, recurse: bool) -> Result<()> {
    let group_id = match is_multiuser {
        true => match get_group_id(PACKIT_GROUP_NAME) {
            Ok(uid) => Some(uid),
            Err(PermissionError::GroupDoesNotExist) => {
                warning!("The 'packit' group does not exist. Consider running 'pit fix' to fix your Packit installation");
                return Err(PermissionError::GroupDoesNotExist);
            },
            Err(e) => return Err(e),
        },
        false => None,
    };

    let metadata = fs::metadata(&path)?;
    set_file_permissions(path, metadata.permissions(), group_id, recurse)
}

/// Sets the permissions of a given directory. It does so recursively if the recurse parameter is true.
/// Could return an IO error.
fn set_file_permissions(path: &PathBuf, mut old_permissions: Permissions, group_id: Option<u32>, recurse: bool) -> Result<()> {
    if let Some(group_id) = group_id {
        set_ownership(path, None, Some(group_id))?;

        old_permissions.set_mode(0o775);
    } else {
        old_permissions.set_mode(0o755);
    }

    fs::set_permissions(&path, old_permissions)?;

    if !recurse || !path.is_dir() {
        return Ok(());
    }

    let dir = fs::read_dir(&path)?;
    for entry in dir {
        let entry = entry?;

        set_file_permissions(&entry.path(), entry.metadata()?.permissions(), group_id, recurse)?;
    }

    Ok(())
}

/// Set ownership of a directory based on the provided user and group id.
/// Could return an IO error.
pub fn set_ownership(path: &PathBuf, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
    // If the path is a symlink, set symlink ownership
    if path.is_symlink() {
        unix::fs::lchown(path, uid, gid)?;
        return Ok(());
    }

    // Set file ownership
    unix::fs::chown(path, uid, gid)?;

    Ok(())
}

/// Gets the group id based on the given name.
pub fn get_group_id(name: &str) -> Result<u32> {
    let c_name = CString::new(name).map_err(PlatformError::from)?;
    let group = unsafe { libc::getgrnam(c_name.as_ptr()) };

    if group.is_null() {
        return Err(PermissionError::GroupDoesNotExist);
    }

    unsafe { Ok((*group).gr_gid) }
}
