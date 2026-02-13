use bytes::Bytes;
use flate2::read::GzDecoder;
use std::{io::Cursor, path::Path};
use tar::Archive;
use thiserror::Error;

use crate::cli::display::ReaderWithProgress;

/// The errors that occur during unpacking.
#[derive(Error, Debug)]
pub enum UnpackError {
    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),

    #[error("The file extension is not supported")]
    ExtensionNotSupported,

    #[error("Cannot get the extension from the file")]
    NoExtension,
}

type Result<T> = core::result::Result<T, UnpackError>;

// Unpacks tar files and saves them to a provided destination directory
pub fn unpack<P: AsRef<Path>>(source_path: &str, bytes: Bytes, destination_directory: P) -> Result<()> {
    let size = bytes.len();
    let cursor = Cursor::new(bytes);

    // Initialize progress bar
    let reader = ReaderWithProgress::new(cursor, size as u64);

    // Get extension from url
    let extension_index = source_path.chars().rev().position(|x| x == '.').ok_or(UnpackError::NoExtension)?;
    let (_, extension) = source_path.split_at(extension_index + 1);

    match extension {
        "gz" => unpack_gz(reader, destination_directory),
        _ => Err(UnpackError::ExtensionNotSupported),
    }
}

fn unpack_gz<P: AsRef<Path>>(reader: ReaderWithProgress<Cursor<Bytes>>, destination_directory: P) -> Result<()> {
    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);
    archive.unpack(destination_directory)?;
    Ok(())
}
