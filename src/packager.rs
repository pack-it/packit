use std::{
    fs::File,
    io::{self, Cursor, Write},
    path::PathBuf,
};

use flate2::{write::GzEncoder, Compression};
use tar::Builder;
use thiserror::Error;

use crate::{
    config::Config,
    installer::types::PackageId,
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, types::Checksum},
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum PackagerError {
    #[error("Cannot get revisions from repository manager.")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error while packaging.")]
    IOError(#[from] std::io::Error),
}

pub fn package(config: &Config, package_id: &PackageId, destination: &PathBuf, revisions: usize) -> Result<(), PackagerError> {
    let install_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

    // Return an error if the destination is not a directory
    if !destination.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotADirectory, "Destination is not a directory."))?;
    }

    // Compress the package
    let mut compressed = compress(&install_directory)?;

    // Calculate checksum for the compressed
    let checksum = Checksum::calculate_checksum(&mut compressed)?;

    // Create the file names
    let filename = format!("{package_id}-{revisions}-{TARGET_ARCHITECTURE}");
    let compressed_filename = format!("{filename}.tar.gz");
    let prepackage_dir = destination.join(compressed_filename);
    let checksum_filename = format!("{filename}.sha256");

    // Store the compressed package and checksum
    let mut compressed_file = File::create(prepackage_dir)?;
    let mut checksum_file = File::create(checksum_filename)?;
    compressed_file.write_all(compressed.get_ref())?;
    checksum_file.write_all(checksum.as_bytes())?;

    Ok(())
}

pub fn compress(source_directory: &PathBuf) -> Result<Cursor<Vec<u8>>, PackagerError> {
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let encoder = GzEncoder::new(cursor, Compression::default());

    // Add the whole directory recursively
    let mut tar = Builder::new(encoder);
    tar.append_dir_all(".", source_directory)?;

    // Finish writing
    let encoder = tar.into_inner()?;
    let encoded = encoder.finish()?;

    Ok(encoded)
}
