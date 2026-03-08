use std::{
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use flate2::{Compression, GzBuilder, write::GzEncoder};
use tar::{Builder, EntryType, Header};
use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::types::PackageId,
    platforms::TargetArchitecture,
    repositories::{error::RepositoryError, types::Checksum},
};

/// The errors that occur during installation.
#[derive(Error, Debug)]
pub enum PackagerError {
    #[error("Cannot parse filename, because it contains invalid unicode")]
    InvalidUnicodeError,

    #[error("Cannot get revisions from repository manager")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error while packaging")]
    IOError(#[from] std::io::Error),
}

pub fn package(config: &Config, package_id: &PackageId, destination: &PathBuf, revisions: usize) -> Result<(), PackagerError> {
    warning!("This is an experimental feature, checksums calculated with the packager for pre-builds might not be accurate.");
    let install_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

    // Return an error if the destination is not a directory
    if !destination.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotADirectory, "Destination is not a directory."))?;
    }

    // Compress the package
    let compressed = compress(&install_directory)?;

    // Calculate checksum for the compressed
    let checksum = Checksum::from_bytes(&compressed);

    // Create the file names
    let target_architecture = TargetArchitecture::current().to_string();
    let filename = format!("{package_id}-{revisions}-{target_architecture}");
    let compressed_filename = format!("{filename}.tar.gz");
    let prepackage_file = destination.join(compressed_filename);
    let checksum_filename = format!("{filename}.sha256");
    let checksum_file = destination.join(checksum_filename);

    // Store the compressed package and checksum
    let mut compressed_file = File::create(prepackage_file)?;
    let mut checksum_file = File::create(checksum_file)?;
    compressed_file.write_all(&compressed)?;
    checksum_file.write_all(checksum.to_string().as_bytes())?;

    Ok(())
}

pub fn compress(source_directory: &PathBuf) -> Result<Vec<u8>, PackagerError> {
    let buffer = Vec::new();
    let encoder = GzBuilder::new().mtime(0).write(buffer, Compression::default());

    // Add the whole directory recursively
    let mut tar_builder = Builder::new(encoder);
    create_normalized_tar(&mut tar_builder, &PathBuf::from("."), source_directory)?;
    tar_builder.finish()?;

    // Finish writing
    let encoder = tar_builder.into_inner()?;
    let encoded = encoder.finish()?;

    Ok(encoded)
}

fn normalized_header(header: &mut Header, data_length: u64, tar_path: &PathBuf, file_path: &PathBuf) -> Result<(), PackagerError> {
    // For symlink do symlink_metadata() instead of metadata()
    let mode = match header.entry_type() == EntryType::Symlink {
        true => fs::symlink_metadata(file_path)?.permissions().mode(),
        false => fs::metadata(file_path)?.permissions().mode(),
    };

    header.set_size(data_length);
    header.set_mode(mode);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_path(tar_path)?;

    Ok(())
}

fn add_file(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<(), PackagerError> {
    // Get file
    let file = File::open(file_path)?;

    // Create regular file header
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Regular);
    normalized_header(&mut header, file.metadata()?.len() as u64, tar_path, file_path)?;

    // Reset header checksum (different from our checksum)
    header.set_cksum();

    builder.append_data(&mut header, tar_path, file)?;

    Ok(())
}

fn add_directory(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<(), PackagerError> {
    // Create directory header
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Directory);
    normalized_header(&mut header, 0, tar_path, file_path)?;

    // Reset header checksum (different from our checksum)
    header.set_cksum();

    builder.append_data(&mut header, tar_path, io::empty())?;

    Ok(())
}

fn add_symlink(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<(), PackagerError> {
    let target = fs::read_link(file_path)?;

    // Create symlink header
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Symlink);
    normalized_header(&mut header, 0, tar_path, file_path)?;

    // Reset header checksum (different from our checksum)
    header.set_cksum();

    builder.append_link(&mut header, tar_path, target)?;

    Ok(())
}

fn create_normalized_tar(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<(), PackagerError> {
    add_directory(builder, tar_path, file_path)?;

    // Get directory entries
    let mut entries = Vec::new();
    for entry in fs::read_dir(file_path)? {
        let entry = entry?;
        entries.push(entry.path());
    }

    // Sort the entries for deterministic behaviour
    entries.sort();

    // Add to tar file recursively
    for entry in entries {
        let filename = entry.file_name().expect("Expected a valid path termination");
        let filename = filename.to_str().ok_or(PackagerError::InvalidUnicodeError)?;

        // Check for symlink first
        if entry.is_symlink() {
            add_symlink(builder, &tar_path.join(filename), &entry)?;
            continue;
        }

        if entry.is_dir() {
            create_normalized_tar(builder, &tar_path.join(filename), &entry)?;
            continue;
        }

        if entry.is_file() {
            add_file(builder, &tar_path.join(filename), &entry)?;
            continue;
        }
    }

    Ok(())
}
