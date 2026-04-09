// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Group does not exist")]
    GroupDoesNotExist,

    #[error("Error while fetching permissions")]
    IOError(#[from] std::io::Error),

    #[error("String contains a nul byte")]
    NulError(#[from] std::ffi::NulError),
}

type Result<T> = core::result::Result<T, PermissionError>;

/// Checks if a path is writable by the current user. Returns true if it is, false otherwise.
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

/// Sets the permissions of packit files
pub use platform::set_packit_permissions;

/// Permissions implementation for Unix platforms.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub mod platform {
    use std::{
        ffi::CString,
        fs::{self, Metadata, Permissions},
        os::unix::{
            self,
            fs::{MetadataExt, PermissionsExt},
        },
        path::PathBuf,
    };

    use crate::cli::display::logging::warning;

    use super::{PermissionError, Result};

    pub const PACKIT_GROUP_NAME: &str = "packit";

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
    use std::{ffi::OsStr, fs::Metadata, os::windows::ffi::OsStrExt, path::PathBuf, ptr};

    use super::Result;

    use windows::{
        Win32::{
            Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GENERIC_ALL, GENERIC_WRITE, HANDLE},
            Security::{
                ACE_REVISION, ACL,
                Authorization::{
                    ConvertSidToStringSidW, EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW, SE_FILE_OBJECT, SetEntriesInAclW,
                    SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_W,
                },
                DACL_SECURITY_INFORMATION, GetTokenInformation, InitializeAcl, NO_INHERITANCE, OWNER_SECURITY_INFORMATION,
                PSECURITY_DESCRIPTOR, PSID, TOKEN_QUERY, TOKEN_USER, TokenUser,
            },
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        },
        core::{PCWSTR, PWSTR},
    };

    pub fn is_writable(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        // TODOs
        Ok(true)
    }

    // TODO: Implement multiuser and ownership
    pub fn set_packit_permissions(path: &PathBuf, _is_multiuser: bool, recurse: bool) -> Result<()> {
        unsafe {
            // TODO: For groups set this sid to a group sid instead
            // Get the current user
            let user_sid = get_current_sid();

            // Get the current owner
            let mut p_owner = PSID(ptr::null_mut());
            let acl = ptr::null_mut();
            let mut p_sd = PSECURITY_DESCRIPTOR(ptr::null_mut());

            let wide_path = PCWSTR(path_to_pcwstr(path).as_ptr());
            let result = GetNamedSecurityInfoW(
                PCWSTR(wide_path.as_ptr()),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                Some(&mut p_owner),
                None,
                Some(acl),
                None,
                &mut p_sd,
            );

            if result.0 != ERROR_SUCCESS.0 {
                // TODO: Return some error, failed to get security info
                panic!("Error while getting security info");
            }

            // Adjust DACL to set permissions
            let mut explicit_access = EXPLICIT_ACCESS_W::default();

            explicit_access.grfAccessPermissions = GENERIC_ALL.0;
            explicit_access.grfAccessMode = GRANT_ACCESS;
            explicit_access.grfInheritance = NO_INHERITANCE;

            // Set the trustee (for which user the entry is meant)
            let mut trustee = TRUSTEE_W::default();
            trustee.TrusteeForm = TRUSTEE_IS_SID;
            trustee.ptstrName = PWSTR(user_sid.0 as *mut u16);
            explicit_access.Trustee = trustee;

            let mut new_acl = ptr::null_mut();
            let error = SetEntriesInAclW(Some(&[explicit_access]), Some(acl as *mut ACL), &mut new_acl);
            if error.0 != ERROR_SUCCESS.0 {
                dbg!(&error);
                panic!("Error while setting ACL entries");
            }

            let wide_path = PCWSTR(path_to_pcwstr(path).as_ptr());
            let error = SetNamedSecurityInfoW(
                wide_path,
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                Some(p_owner),
                None,
                Some(new_acl as *mut ACL),
                None,
            );

            if error.0 != ERROR_SUCCESS.0 {
                // TODO: Return some error
                panic!("Error while setting security info");
            }

            Ok(())
        }
    }

    fn get_current_sid() -> PSID {
        unsafe {
            let mut token: HANDLE = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).unwrap();

            // Get the size of the token information for the buffer
            let mut size: u32 = 0; // TODO: Not sure if type is needed
            match GetTokenInformation(token, TokenUser, None, 0, &mut size) {
                Ok(_) => {},
                Err(_) => {},
            }

            let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
            GetTokenInformation(token, TokenUser, Some(buffer.as_mut_ptr() as *mut _), size, &mut size).unwrap();

            let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);

            // dbg!(&token_user.User.Sid);
            // let mut sid_str = PWSTR::null();
            // ConvertSidToStringSidW(token_user.User.Sid.clone(), &mut sid_str).unwrap();
            // dbg!(sid_str.to_string().unwrap());

            token_user.User.Sid
        }
    }

    // TODO: Probably wrong, use path.to_str() then use w!
    fn path_to_pcwstr(path: &PathBuf) -> Vec<u16> {
        OsStr::new(path)
            .encode_wide()
            .chain(Some(0)) // null-termination
            .collect()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use super::Result;

    pub(super) fn is_writable(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        panic!("Cannot check write permissions for target, target is not supported.");
    }

    pub fn set_packit_permissions(_path: &PathBuf, _is_multiuser: bool, _recurse: bool) -> Result<()> {
        panic!("Cannot set ownership for target, target is not supported.");
    }
}
