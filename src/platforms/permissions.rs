use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Error while fetching permissions")]
    IOError(#[from] std::io::Error),

    #[error("Group does not exist")]
    GroupDoesNotExist,

    #[error("String contains a nul byte")]
    NulError(#[from] std::ffi::NulError),
}

type Result<T> = core::result::Result<T, PermissionError>;

pub fn is_writable(path: &PathBuf) -> Result<bool> {
    if !fs::exists(path)? {
        return Ok(false);
    }

    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();

    // Check if path is read only
    if permissions.readonly() {
        return Ok(false);
    }

    // Use platform specific writable checks
    platform::is_writable(path, metadata)
}

pub use platform::set_packit_ownership;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub mod platform {
    use std::{
        ffi::CString,
        fs::Metadata,
        os::unix::fs::{self, MetadataExt},
        path::PathBuf,
    };

    use crate::cli::display::logging::warning;

    use super::{PermissionError, Result};

    pub(super) fn is_writable(_path: &PathBuf, metadata: Metadata) -> Result<bool> {
        let mode = metadata.mode();

        // Check if path is writable for everyone
        if mode & 0o002 != 0 {
            return Ok(true);
        }

        // Check if path is writable for group
        let current_group = unsafe { libc::getegid() };
        let group = metadata.gid();
        if current_group == group && mode & 0o020 != 0 {
            return Ok(true);
        }

        // Check if path is writable for user
        let current_user = unsafe { libc::geteuid() };
        let user = metadata.uid();
        if current_user == user && mode & 0o200 != 0 {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn set_packit_ownership(path: &PathBuf) -> Result<()> {
        let packit_group = match get_group_id("packit") {
            Ok(uid) => uid,
            Err(e) => {
                if matches!(e, PermissionError::GroupDoesNotExist) {
                    warning!("The 'packit' group does not exist. Please run 'pit fix' to fix your Packit installation");
                    // TODO: Add group check to verifier
                }
                return Err(e);
            },
        };

        set_ownership(path, None, Some(packit_group))?;

        Ok(())
    }

    pub fn set_ownership(path: &PathBuf, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
        // If the path is a symlink, set symlink ownership
        if path.is_symlink() {
            fs::lchown(path, uid, gid)?;
            return Ok(());
        }

        // Set file ownership
        fs::chown(path, uid, gid)?;

        Ok(())
    }

    pub fn get_group_id(name: &str) -> Result<u32> {
        let c_name = CString::new(name)?;
        let group = unsafe { libc::getgrnam(c_name.as_ptr()) };

        if group.is_null() {
            return Err(PermissionError::GroupDoesNotExist);
        }

        unsafe {
            return Ok((*group).gr_gid);
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use super::Result;

    pub fn is_writable_specific(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        //TODO
        Ok(false)
    }

    pub fn set_packit_ownership(path: &PathBuf) -> Result<()> {
        todo!()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use super::Result;

    pub fn is_writable_specific(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        panic!("Cannot check write permissions for target, target is not supported.");
    }

    pub fn set_packit_ownership(path: &PathBuf) -> Result<()> {
        panic!("Cannot set ownership for target, target is not supported.");
    }
}
