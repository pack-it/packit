use std::io::Cursor;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::installer::error::{InstallerError, Result};

/// Unpacks tar files and saves them to the provided install directory.
pub fn unpack(response: reqwest::blocking::Response, install_directory: &String) -> Result<()> {
    let bytes = match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => return Err(InstallerError::RequestError(e)),
    };

    let cursor = Cursor::new(bytes);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    match archive.unpack(install_directory) {
        Ok(_) => {}
        Err(e) => return Err(InstallerError::UnpackError(e)),
    };
    Ok(())
}
