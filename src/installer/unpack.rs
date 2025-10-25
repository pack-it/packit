use bytes::Bytes;
use flate2::read::GzDecoder;
use std::io::Cursor;
use tar::Archive;

use crate::{cli::display::ReaderWithProgress, installer::error::Result};

// Unpacks tar files and saves them to a provided install directory
pub fn unpack(bytes: Bytes, install_directory: &String) -> Result<()> {
    let size = bytes.len();
    let cursor = Cursor::new(bytes);

    // Initialize progress bar
    let reader = ReaderWithProgress::new(cursor, size as u64);

    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);
    archive.unpack(install_directory)?;
    Ok(())
}
