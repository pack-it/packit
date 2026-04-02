// SPDX-License-Identifier: GPL-3.0-only
use bytes::Bytes;
use flate2::read::GzDecoder;
use std::{io::Cursor, path::Path};
use tar::Archive;
use thiserror::Error;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::{cli::display::ReaderWithProgress, installer::types::PackageName};

/// The errors that occur during unpacking.
#[derive(Error, Debug)]
pub enum UnpackError {
    #[error("The file extension is not supported")]
    ExtensionNotSupported,

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),

    #[error("Error while unpacking")]
    ZipError(#[from] zip::result::ZipError),
}

type Result<T> = core::result::Result<T, UnpackError>;

/// The different supported ArchiveExtensions.
pub enum ArchiveExtension {
    GZ,
    ZIP,
    XZ,
    Unknown,
}

impl ArchiveExtension {
    /// Creates an ArchiveExtension from a path.
    pub fn from_path(path: &str) -> Self {
        let extension_index = match path.chars().rev().position(|x| x == '.') {
            Some(index) => index,
            None => return Self::Unknown,
        };
        let (_, extension) = path.split_at(path.len() - extension_index);

        match extension.to_lowercase().as_str() {
            "gz" | "tgz" => Self::GZ,
            "zip" => Self::ZIP,
            "xz" => Self::XZ,
            _ => Self::Unknown,
        }
    }
}

// Unpacks files and saves them to the provided destination directory.
pub fn unpack<P: AsRef<Path>>(
    package: &PackageName,
    extension: ArchiveExtension,
    bytes: Bytes,
    destination_directory: P,
    keep_timestamp: bool,
) -> Result<()> {
    let size = bytes.len();
    let cursor = Cursor::new(bytes);

    // Initialize progress bar
    let bar_prefix = format!("Unpacking {package}");
    let reader = ReaderWithProgress::new(cursor, size as u64, bar_prefix);

    match extension {
        ArchiveExtension::GZ => unpack_gz(reader, destination_directory, keep_timestamp),
        ArchiveExtension::XZ => unpack_xz(reader, destination_directory, keep_timestamp),
        ArchiveExtension::ZIP => unpack_zip(reader, destination_directory),
        _ => Err(UnpackError::ExtensionNotSupported),
    }
}

/// Unpacks gz archives into the provided destination directory.
/// Could return an IO error.
fn unpack_gz<P: AsRef<Path>>(reader: ReaderWithProgress<Cursor<Bytes>>, destination_directory: P, keep_timestamp: bool) -> Result<()> {
    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);
    archive.set_preserve_mtime(keep_timestamp);
    archive.unpack(destination_directory)?;

    Ok(())
}

/// Unpacks xz archives into the provided destination directory.
/// Could return an IO error.
fn unpack_xz<P: AsRef<Path>>(reader: ReaderWithProgress<Cursor<Bytes>>, destination_directory: P, keep_timestamp: bool) -> Result<()> {
    let tar = XzDecoder::new(reader);
    let mut archive = Archive::new(tar);
    archive.set_preserve_mtime(keep_timestamp);
    archive.unpack(destination_directory)?;

    Ok(())
}

/// Unpacks zip archives into the provided destination directory.
/// Could return an IO error.
fn unpack_zip<P: AsRef<Path>>(reader: ReaderWithProgress<Cursor<Bytes>>, destination_directory: P) -> Result<()> {
    let mut archive = ZipArchive::new(reader)?;
    archive.extract(destination_directory)?;

    Ok(())
}
