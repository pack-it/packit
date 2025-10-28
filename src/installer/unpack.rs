use bytes::Bytes;
use flate2::read::GzDecoder;
use std::{io::Cursor, path::Path};
use tar::Archive;

use crate::{cli::ReaderWithProgress, installer::error::Result};

// Unpacks tar files and saves them to a provided install directory
pub fn unpack<P: AsRef<Path>>(bytes: Bytes, install_directory: P) -> Result<()> {
    let size = bytes.len();
    let cursor = Cursor::new(bytes);

    // Initialize progress bar
    let reader = ReaderWithProgress::new(cursor, size as u64);

    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);
    archive.unpack(install_directory)?;
    Ok(())
}
