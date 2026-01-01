use std::path::Path;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn create_symlink(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(source, destination)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn create_symlink(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    match source.is_dir() {
        true => std::os::windows::fs::symlink_dir(source, destination)?,
        false => std::os::windows::fs::symlink_file(source, destination)?,
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn create_symlink(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    panic!("Cannot create link for target, target is not supported.");
}

pub fn remove_symlink(symlink: &Path) -> Result<(), std::io::Error> {
    std::fs::remove_file(symlink)?;

    Ok(())
}
