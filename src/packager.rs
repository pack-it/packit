use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use flate2::{write::GzEncoder, Compression};
use tar::Builder;
use thiserror::Error;

use crate::{
    config::Config,
    installer::types::PackageId,
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, manager::RepositoryManager, types::Checksum},
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum PackagerError {
    #[error("Cannot get revisions from repository manager.")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error while packaging.")]
    IOError(#[from] std::io::Error),
}

pub fn package(config: &Config, package_id: &PackageId, destination: &PathBuf, manager: &RepositoryManager) -> Result<(), PackagerError> {
    let install_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

    // Return an error if the destination is a directory
    if !destination.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotADirectory, "Destination is not a directory."))?;
    }

    // Get the number of revisions of the package
    let (_, package_meta) = manager.read_package(&package_id.name)?;
    let revisions = package_meta.revisions.len();

    // Create pre-package filename
    let filename = format!("{package_id}-{revisions}-{TARGET_ARCHITECTURE}.tar.gz");
    let compressed_filename = format!("{filename}.tar.gz");
    let prepackage_dir = destination.join(compressed_filename);

    let result = File::create(&prepackage_dir)?;
    let encoder = GzEncoder::new(result, Compression::default());

    // Add the whole directory recursively
    let mut tar = Builder::new(encoder);
    tar.append_dir_all(".", install_directory)?;

    // Finish writing
    let encoder = tar.into_inner()?;
    encoder.finish()?;

    // Calculate checksum for the compressed
    let mut compressed = File::open(&prepackage_dir)?;
    let checksum = Checksum::calculate_checksum(&mut compressed)?;

    // Store checksum
    let checksum_filename = format!("{filename}.sha256");
    let mut checksum_file = File::create(checksum_filename)?;
    checksum_file.write_all(checksum.as_bytes())?;

    Ok(())
}
