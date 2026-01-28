use std::{fs, path::PathBuf};

type Result<T> = core::result::Result<T, std::io::Error>;

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

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod platform {
    use std::{fs::Metadata, os::unix::fs::MetadataExt, path::PathBuf};

    use crate::platforms::permissions::Result;

    #[link(name = "c")]
    extern "C" {
        fn geteuid() -> u32;
        fn getegid() -> u32;
    }

    pub fn is_writable(_path: &PathBuf, metadata: Metadata) -> Result<bool> {
        let mode = metadata.mode();

        // Check if path is writable for everyone
        if mode & 0o002 != 0 {
            return Ok(true);
        }

        // Check if path is writable for group
        let current_group = unsafe { getegid() };
        let group = metadata.gid();
        if current_group == group && mode & 0o020 != 0 {
            return Ok(true);
        }

        // Check if path is writable for user
        let current_user = unsafe { geteuid() };
        let user = metadata.uid();
        if current_user == user && mode & 0o200 != 0 {
            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use crate::platforms::permissions::Result;

    pub fn is_writable_specific(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        Ok(false)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use std::{fs::Metadata, path::PathBuf};

    use crate::platforms::permissions::Result;

    pub fn is_writable_specific(_path: &PathBuf, _metadata: Metadata) -> Result<bool> {
        panic!("Cannot check write permissions for target, target is not supported.");
    }
}
