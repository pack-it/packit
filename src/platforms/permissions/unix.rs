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
    utils::ioerror::IOResultExt,
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
    // SAFETY: getegid is always successful, has no safety preconditions and thus cannot fail
    let current_group = unsafe { libc::getegid() };
    let group = metadata.gid();
    if current_group == group && mode & 0o020 != 0 {
        return Ok(true);
    }

    // Check if path is writable for current user
    // SAFETY: geteuid is always successful, has no safety preconditions and thus cannot fail
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

    let metadata = fs::metadata(path).err_with_path("read metadata of", path)?;
    set_file_permissions(path, metadata.permissions(), group_id, recurse)
}

/// Checks if the packit group exists.
pub fn does_packit_group_exist() -> Result<bool> {
    match get_group_id(PACKIT_GROUP_NAME) {
        Ok(_) => Ok(true),
        Err(PermissionError::GroupDoesNotExist) => Ok(false),
        Err(e) => Err(e),
    }
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

    fs::set_permissions(path, old_permissions).err_with_path("set permissions of", path)?;

    if !recurse || !path.is_dir() {
        return Ok(());
    }

    let dir = fs::read_dir(path).err_with_path("read", path)?;
    for entry in dir {
        let entry = entry.err_with_path("iterate", path)?;

        let metadata = entry.metadata().err_with_path("read metadata of", entry.path())?;
        set_file_permissions(&entry.path(), metadata.permissions(), group_id, recurse)?;
    }

    Ok(())
}

/// Set ownership of a directory based on the provided user and group id.
/// Could return an IO error.
pub fn set_ownership(path: &PathBuf, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
    // If the path is a symlink, set symlink ownership
    if path.is_symlink() {
        unix::fs::lchown(path, uid, gid).err_with_path("change ownership of", path)?;
        return Ok(());
    }

    // Set file ownership
    unix::fs::chown(path, uid, gid).err_with_path("change ownership of", path)?;

    Ok(())
}

/// Gets the group id based on the given name.
pub fn get_group_id(name: &str) -> Result<u32> {
    let c_name = CString::new(name).map_err(PlatformError::from)?;

    // SAFETY: `c_name` is valid nul terminated string created by `CString`
    // SAFETY: getgrnam returns either a valid pointer or a null pointer, which we check
    let group = unsafe { libc::getgrnam(c_name.as_ptr()) };
    if group.is_null() {
        return Err(PermissionError::GroupDoesNotExist);
    }

    // SAFETY: the group pointer is not null, because we check for this and return an error if it is
    // SAFETY: no free is needed, since the getgrnam man page explicitly forbids this
    unsafe { Ok((*group).gr_gid) }
}
