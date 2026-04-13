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

    #[error("Error during platform specific operations")]
    PlatformError(#[from] PlatformError),
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
pub use self::platform::set_packit_permissions;

pub use self::platform::PlatformError;

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

    #[derive(Error, Debug)]
    pub enum PlatformError {}

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
    use std::{
        ffi::OsStr,
        fs::{self, Metadata},
        os::windows::ffi::OsStrExt,
        path::PathBuf,
        ptr,
    };

    use crate::{cli::display::logging::warning, platforms::permissions::PermissionError};

    use super::Result;

    use thiserror::Error;
    use windows::{
        Win32::{
            Foundation::{
                CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NONE_MAPPED, ERROR_SUCCESS, GENERIC_ALL, GENERIC_WRITE, HANDLE, HLOCAL, LUID,
                LocalFree,
            },
            Security::{
                ACL, AccessCheck, AdjustTokenPrivileges,
                Authorization::{
                    EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SetEntriesInAclW,
                    SetNamedSecurityInfoW, TRUSTEE_IS_GROUP, TRUSTEE_IS_SID, TRUSTEE_IS_USER, TRUSTEE_W,
                },
                CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION, DuplicateTokenEx, GENERIC_MAPPING, GROUP_SECURITY_INFORMATION,
                GetTokenInformation, LUID_AND_ATTRIBUTES, LookupAccountNameW, LookupPrivilegeValueW, MapGenericMask, NO_INHERITANCE,
                OBJECT_INHERIT_ACE, OWNER_SECURITY_INFORMATION, PRIVILEGE_SET, PSECURITY_DESCRIPTOR, PSID, SE_PRIVILEGE_ENABLED,
                SID_NAME_USE, SecurityImpersonation, TOKEN_ACCESS_MASK, TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE, TOKEN_IMPERSONATE,
                TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER, TokenImpersonation, TokenUser,
            },
            Storage::FileSystem::{FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE},
            System::{
                Memory::{LMEM_FIXED, LocalAlloc},
                SystemServices::MAXIMUM_ALLOWED,
                Threading::{GetCurrentProcess, OpenProcessToken},
            },
        },
        core::{BOOL, HSTRING, PCWSTR, PWSTR, w},
    };

    #[derive(Error, Debug)]
    pub enum PlatformError {
        #[error("Security info error with code {code}. {message}")]
        SecurityInfoError {
            message: String,
            code: u32,
        },

        #[error("Error while interacting with windows API")]
        WindowsAPIError(#[from] windows::core::Error),
    }

    /// Checks if the given directory is writable by the current user. Returns true if it is, false if not.
    /// Could return a `PlatformError::SecurityInfoError` or a `PlatformError::WindowsAPIError` error.
    pub fn is_writable(path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        let wide_path_buffer = path_to_pcwstr(path);
        let wide_path = PCWSTR(wide_path_buffer.as_ptr());
        let (_, _, _, _, security_descriptor) = get_security_info(wide_path)?;

        unsafe {
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_DUPLICATE | TOKEN_IMPERSONATE, &mut token)
                .map_err(PlatformError::WindowsAPIError)?;

            let mut impersonation_token = HANDLE::default();
            DuplicateTokenEx(
                token,
                TOKEN_ACCESS_MASK(MAXIMUM_ALLOWED),
                None,
                SecurityImpersonation,
                TokenImpersonation,
                &mut impersonation_token,
            )
            .map_err(PlatformError::WindowsAPIError)?;

            let mut desired_access = GENERIC_WRITE;
            const MAPPING: GENERIC_MAPPING = GENERIC_MAPPING {
                GenericRead: FILE_GENERIC_READ.0,
                GenericWrite: FILE_GENERIC_WRITE.0,
                GenericExecute: FILE_GENERIC_EXECUTE.0,
                GenericAll: FILE_ALL_ACCESS.0,
            };

            MapGenericMask(&mut desired_access.0, &MAPPING);

            let mut granted_access: u32 = 0;
            let mut access_status = BOOL(0);
            let mut privilege_length: u32 = 0;
            let result = AccessCheck(
                security_descriptor,
                impersonation_token,
                desired_access.0,
                &MAPPING,
                None,
                &mut privilege_length,
                &mut granted_access,
                &mut access_status,
            );

            match result {
                Ok(_) => warning!("Unexpected succes on first call"),
                Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => {},
                Err(e) => return Err(PlatformError::WindowsAPIError(e))?,
            }

            let mut privilege_buffer = vec![0u8; privilege_length as usize];
            let privilege = privilege_buffer.as_mut_ptr() as *mut PRIVILEGE_SET;
            let mut granted_access: u32 = 0;
            let mut access_status = BOOL(0);
            AccessCheck(
                security_descriptor,
                impersonation_token,
                desired_access.0,
                &MAPPING,
                Some(privilege),
                &mut privilege_length,
                &mut granted_access,
                &mut access_status,
            )
            .map_err(PlatformError::WindowsAPIError)?;

            // Free the token handles
            CloseHandle(token).map_err(PlatformError::WindowsAPIError)?;
            CloseHandle(impersonation_token).map_err(PlatformError::WindowsAPIError)?;

            Ok(access_status.as_bool())
        }
    }

    /// Sets the permissions for the current user and for the packit group if multiuser is enabled.
    /// If multiuser mode is enabled it will also set the ownership for the entire packit group.
    /// Could return a `PlatformError::SecurityInfoError`, a `PlatformError::WindowsAPIError` or a `PermissionError::GroupDoesNotExist` error.
    pub fn set_packit_permissions(path: &PathBuf, is_multiuser: bool, recurse: bool) -> Result<()> {
        // Get the current sid
        let (sid, _) = match is_multiuser {
            true => get_group_sid()?,
            false => get_user_sid()?,
        };

        // This assumes that the current user already has ownership
        if is_multiuser {
            set_group_ownership(path, sid, recurse)?;
        }

        let wide_path_buffer = path_to_pcwstr(path);
        let wide_path = PCWSTR(wide_path_buffer.as_ptr());
        let (_, _, acl, _, security_descriptor) = get_security_info(wide_path)?;

        unsafe {
            let inheritance_state = match recurse {
                true => OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                false => NO_INHERITANCE,
            };

            // Set the trustee (for which user the entry is meant)
            let mut trustee = TRUSTEE_W::default();
            trustee.ptstrName = PWSTR(sid.0 as *mut _);
            trustee.TrusteeType = if is_multiuser { TRUSTEE_IS_GROUP } else { TRUSTEE_IS_USER };
            trustee.TrusteeForm = TRUSTEE_IS_SID;
            trustee.pMultipleTrustee = ptr::null_mut();
            trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;

            // Adjust DACL to set permissions
            let mut explicit_access = EXPLICIT_ACCESS_W::default();
            explicit_access.grfAccessPermissions = GENERIC_ALL.0;
            explicit_access.grfAccessMode = GRANT_ACCESS;
            explicit_access.grfInheritance = inheritance_state;
            explicit_access.Trustee = trustee;

            // Set the ACL entries by passing a list of new entries and the old acl.
            // This will be combined into the new acl.
            let mut new_acl = ptr::null_mut();
            let result = SetEntriesInAclW(Some(&[explicit_access]), Some(acl as *mut ACL), &mut new_acl);
            if result.0 != ERROR_SUCCESS.0 {
                return Err(PlatformError::SecurityInfoError {
                    message: "Settings ACL entries failed".to_string(),
                    code: result.0,
                })?;
            }

            // Set the new ACL
            let result = SetNamedSecurityInfoW(
                wide_path,
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                Some(sid),
                None,
                Some(new_acl as *mut ACL),
                None,
            );

            if result.0 != ERROR_SUCCESS.0 {
                return Err(PlatformError::SecurityInfoError {
                    message: "Setting security info failed".to_string(),
                    code: result.0,
                })?;
            }

            // Free the security descriptor and acl
            LocalFree(Some(HLOCAL(security_descriptor.0)));
            LocalFree(Some(HLOCAL(new_acl as *mut _)));

            // Free the sid if the sid is from the multiuser
            // Assumes that the multiuser sid will be created with local allocation
            if is_multiuser {
                LocalFree(Some(HLOCAL(sid.0)));
            }

            Ok(())
        }
    }

    /// Enables a given privilege.
    /// Could return a `PlatformError::WindowsAPIError` error.
    fn enable_privilege(name: &str) -> Result<()> {
        unsafe {
            // Get a handle for the current process
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token)
                .map_err(PlatformError::WindowsAPIError)?;

            // Get the unique id for the given privilege name
            let mut luid = LUID::default();
            LookupPrivilegeValueW(None, &HSTRING::from(name), &mut luid).map_err(PlatformError::WindowsAPIError)?;

            let token_privileges = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };

            AdjustTokenPrivileges(token, false, Some(&token_privileges), 0, None, None).map_err(PlatformError::WindowsAPIError)?;

            // Free the token handle
            CloseHandle(token).map_err(PlatformError::WindowsAPIError)?;
        }

        Ok(())
    }

    /// Recursively set the ownership for a given path (if recurse is true).
    /// Could return a `PlatformError::SecurityInfoError` or a `PlatformError::WindowsAPIError` error.
    fn set_group_ownership(path: &PathBuf, sid: PSID, recurse: bool) -> Result<()> {
        let wide_path_buffer = path_to_pcwstr(path);
        let wide_path = PCWSTR(wide_path_buffer.as_ptr());

        unsafe {
            // Enable the correct privileges before changing the ownership of the path
            enable_privilege("SeRestorePrivilege")?;
            enable_privilege("SeTakeOwnershipPrivilege")?;

            // Set the ownership for the given `PSID`
            let result = SetNamedSecurityInfoW(wide_path, SE_FILE_OBJECT, OWNER_SECURITY_INFORMATION, Some(sid), None, None, None);
            if result.0 != ERROR_SUCCESS.0 {
                return Err(PlatformError::SecurityInfoError {
                    message: "Setting security info for group ownership failed".to_string(),
                    code: result.0,
                })?;
            }
        };

        if !recurse || !path.is_dir() {
            return Ok(());
        }

        // Recurse the directory
        for entry in fs::read_dir(&path)? {
            let entry = entry?;

            set_group_ownership(&entry.path(), sid, recurse)?;
        }

        Ok(())
    }

    /// Gets the security info for a certain path.
    /// Could return a `PlatformError::SecurityInfoError` or a `PlatformError::WindowsAPIError` error.
    fn get_security_info(wide_path: PCWSTR) -> Result<(PSID, PSID, *mut ACL, *mut ACL, PSECURITY_DESCRIPTOR)> {
        unsafe {
            // Get the current owner, the acl of the path and the security descriptor
            let mut current_owner_sid = PSID(ptr::null_mut());
            let mut current_group_sid = PSID(ptr::null_mut());
            let mut acl = ptr::null_mut();
            let mut sacl = ptr::null_mut();
            let mut security_descriptor = PSECURITY_DESCRIPTOR(ptr::null_mut());
            let result = GetNamedSecurityInfoW(
                wide_path,
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
                Some(&mut current_owner_sid),
                Some(&mut current_group_sid),
                Some(&mut acl),
                Some(&mut sacl),
                &mut security_descriptor,
            );

            if result.0 != ERROR_SUCCESS.0 {
                return Err(PlatformError::SecurityInfoError {
                    message: "Getting security info failed".to_string(),
                    code: result.0,
                })?;
            }

            Ok((current_owner_sid, current_group_sid, acl, sacl, security_descriptor))
        }
    }

    /// Gets the user `PSID` from the current user.
    /// Could return a `PlatformError::WindowsAPIError` error.
    fn get_user_sid() -> Result<(PSID, Vec<u16>)> {
        unsafe {
            // Create a handle for the current process
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).map_err(PlatformError::WindowsAPIError)?;

            // Get the size of the token information for the buffer
            let mut sid_size: u32 = 0;
            match GetTokenInformation(token, TokenUser, None, 0, &mut sid_size) {
                Ok(_) => warning!("Unexpected succes on first call"),
                Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => {},
                Err(e) => return Err(PlatformError::WindowsAPIError(e))?,
            }

            // Fill the sid buffer
            let mut sid_buffer = vec![0u16; sid_size as usize];
            GetTokenInformation(token, TokenUser, Some(sid_buffer.as_mut_ptr() as *mut _), sid_size, &mut sid_size)
                .map_err(PlatformError::WindowsAPIError)?;

            let token_user = &*(sid_buffer.as_ptr() as *const TOKEN_USER);

            // Free the token handle
            CloseHandle(token).map_err(PlatformError::WindowsAPIError)?;

            Ok((token_user.User.Sid, sid_buffer))
        }
    }

    /// Gets the group `PSID` for the packit group.
    /// Could return a `PlatformError::WindowsAPIError` or a `PermissionError::GroupDoesNotExist` error.
    fn get_group_sid() -> Result<(PSID, Vec<u16>)> {
        unsafe {
            let mut sid_size: u32 = 0;
            let mut domain_size: u32 = 0;
            let mut sid_type = SID_NAME_USE::default();

            // Do the first call to get the sid and domain buffer sizes
            let result = LookupAccountNameW(None, w!("packit"), None, &mut sid_size, None, &mut domain_size, &mut sid_type);

            // Ignore buffer size errors, only return other errors
            match result {
                Ok(_) => warning!("Unexpected succes on first call"),
                Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => {},
                Err(e) => return Err(PlatformError::WindowsAPIError(e))?,
            }

            // Fill the sid and domain buffer
            let sid = PSID(LocalAlloc(LMEM_FIXED, sid_size as usize).map_err(PlatformError::WindowsAPIError)?.0);
            let mut domain_buffer = vec![0u16; domain_size as usize];
            let domain = PWSTR(domain_buffer.as_mut_ptr() as *mut _);
            let result = LookupAccountNameW(
                None,
                w!("packit"),
                Some(sid),
                &mut sid_size,
                Some(domain),
                &mut domain_size,
                &mut sid_type,
            );

            // Explicitly return a `PermissionError::GroupDoesNotExist` error when the group doesn't exist
            match result {
                Ok(_) => Ok((sid, domain_buffer)),
                Err(e) if e.code() == ERROR_NONE_MAPPED.to_hresult() => return Err(PermissionError::GroupDoesNotExist),
                Err(e) => return Err(PlatformError::WindowsAPIError(e))?,
            }
        }
    }

    /// Converts a given path to a wide string buffer (with null termination)
    fn path_to_pcwstr(path: &PathBuf) -> Vec<u16> {
        OsStr::new(path)
            .encode_wide()
            .chain(Some(0)) // Null termination
            .collect()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use super::Result;

    #[derive(Error, Debug)]
    pub enum PlatformError {}

    pub(super) fn is_writable(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        panic!("Cannot check write permissions for target, target is not supported.");
    }

    pub fn set_packit_permissions(_path: &PathBuf, _is_multiuser: bool, _recurse: bool) -> Result<()> {
        panic!("Cannot set ownership for target, target is not supported.");
    }
}
