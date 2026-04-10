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
            Foundation::{ERROR_SUCCESS, GENERIC_ALL, HANDLE},
            Security::{
                ACL,
                Authorization::{
                    ConvertSidToStringSidW, EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW, SE_FILE_OBJECT, SetEntriesInAclW,
                    SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_W,
                },
                DACL_SECURITY_INFORMATION, GetTokenInformation, LookupAccountNameW, MakeAbsoluteSD, NO_INHERITANCE,
                OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, SID_NAME_USE, SetSecurityDescriptorOwner, TOKEN_QUERY, TOKEN_USER,
                TokenUser,
            },
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        },
        core::{PCWSTR, PWSTR, w},
    };

    pub fn is_writable(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        // TODOs
        Ok(true)
    }

    pub fn set_packit_permissions(path: &PathBuf, is_multiuser: bool, recurse: bool) -> Result<()> {
        unsafe {
            // Get the current sid
            let sid = match is_multiuser {
                true => get_group_sid(),
                false => get_user_sid(),
            };

            // Get the current owner
            let mut current_owner_sid = PSID(ptr::null_mut());
            let acl = ptr::null_mut();
            let mut security_descriptor = PSECURITY_DESCRIPTOR(ptr::null_mut());

            let wide_path = PCWSTR(path_to_pcwstr(path).as_ptr());
            let result = GetNamedSecurityInfoW(
                PCWSTR(wide_path.as_ptr()),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                Some(&mut current_owner_sid),
                None,
                Some(acl),
                None,
                &mut security_descriptor,
            );

            if result.0 != ERROR_SUCCESS.0 {
                // TODO: Return some error, failed to get security info
                panic!("Error while getting security info");
            }

            // This assumes that the current user already has ownership
            if is_multiuser {
                set_group_ownership(security_descriptor, sid);
            }

            // Adjust DACL to set permissions
            let mut explicit_access = EXPLICIT_ACCESS_W::default();

            explicit_access.grfAccessPermissions = GENERIC_ALL.0;
            explicit_access.grfAccessMode = GRANT_ACCESS;
            explicit_access.grfInheritance = NO_INHERITANCE;

            // Set the trustee (for which user the entry is meant)
            let mut trustee = TRUSTEE_W::default();
            trustee.TrusteeForm = TRUSTEE_IS_SID;
            trustee.ptstrName = PWSTR(sid.0 as *mut u16);
            explicit_access.Trustee = trustee;

            let mut new_acl = ptr::null_mut();
            let error = SetEntriesInAclW(Some(&[explicit_access]), Some(acl as *mut ACL), &mut new_acl);
            if error.0 != ERROR_SUCCESS.0 {
                panic!("Error while setting ACL entries");
            }

            let wide_path = PCWSTR(path_to_pcwstr(path).as_ptr());
            let error = SetNamedSecurityInfoW(
                wide_path,
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                Some(sid), // TODO: Check if sid it correct for owner (should be)
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

    fn set_group_ownership(security_descriptor: PSECURITY_DESCRIPTOR, sid: PSID) {
        unsafe {
            let mut absolute_size = 0;
            let mut dacl_size = 0;
            let mut sacl_size = 0;
            let mut owner_size = 0;
            let mut group_size = 0;

            let _ = MakeAbsoluteSD(
                security_descriptor,
                None,
                &mut absolute_size,
                None,
                &mut dacl_size,
                None,
                &mut sacl_size,
                None,
                &mut owner_size,
                None,
                &mut group_size,
            );

            let mut absolute_buffer = vec![0u8; absolute_size as usize];
            let absolute_security_descriptor = PSECURITY_DESCRIPTOR(absolute_buffer.as_mut_ptr() as *mut _);
            let mut dacl_buffer = vec![0u8; dacl_size as usize];
            let mut sacl_buffer = vec![0u8; sacl_size as usize];
            let mut owner_buffer = vec![0u8; owner_size as usize];
            let owner = PSID(owner_buffer.as_mut_ptr() as *mut _);
            let mut group_buffer = vec![0u8; group_size as usize];
            let group = PSID(group_buffer.as_mut_ptr() as *mut _);
            MakeAbsoluteSD(
                security_descriptor,
                Some(absolute_security_descriptor),
                &mut absolute_size,
                Some(dacl_buffer.as_mut_ptr() as *mut _),
                &mut dacl_size,
                Some(sacl_buffer.as_mut_ptr() as *mut _),
                &mut sacl_size,
                Some(owner),
                &mut owner_size,
                Some(group),
                &mut group_size,
            )
            .unwrap();

            SetSecurityDescriptorOwner(absolute_security_descriptor, Some(sid), false).unwrap()
        };
    }

    fn get_user_sid() -> PSID {
        unsafe {
            let mut token: HANDLE = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).unwrap();

            // Get the size of the token information for the buffer
            // TODO: Handle non buffer errors
            let mut size: u32 = 0;
            let _ = GetTokenInformation(token, TokenUser, None, 0, &mut size);

            let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
            GetTokenInformation(token, TokenUser, Some(buffer.as_mut_ptr() as *mut _), size, &mut size).unwrap();

            let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);

            token_user.User.Sid
        }
    }

    fn get_group_sid() -> PSID {
        unsafe {
            let mut sid_size: u32 = 0;
            let mut domain_size: u32 = 0;
            let mut _sid_type = SID_NAME_USE::default();

            // TODO: Handle non buffer errors
            let _ = LookupAccountNameW(
                PCWSTR::null(),
                w!("packit"),
                None,
                &mut sid_size,
                None,
                &mut domain_size,
                &mut _sid_type,
            );

            //let sid = PSID::default();
            let mut sid = vec![0u8; sid_size as usize];
            let psid = PSID(sid.as_mut_ptr() as *mut _);
            let mut domain = vec![0u16; domain_size as usize];
            let domain_str = PWSTR(domain.as_mut_ptr() as *mut _);
            LookupAccountNameW(
                PCWSTR::null(),
                w!("packit"),
                Some(psid),
                &mut sid_size,
                Some(domain_str),
                &mut domain_size,
                &mut _sid_type,
            )
            .unwrap();

            psid
        }
    }

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
