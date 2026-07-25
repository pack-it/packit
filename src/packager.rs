// SPDX-License-Identifier: GPL-3.0-only
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use flate2::{Compression, GzBuilder, write::GzEncoder};
use tar::{Builder, EntryType, Header};
use thiserror::Error;

use crate::{
    config::Config,
    installer::types::PackageId,
    platforms::TargetArchitecture,
    repositories::types::{Checksum, FileSize, PrebuildFileMeta},
    utils::ioerror::{self, IOResultExt},
};

/// The errors that occur while packaging a package.
#[derive(Error, Debug)]
pub enum PackagerError {
    #[error("The destination directory '{}' does not exist or is not a directory", path.display())]
    InvalidDestination {
        path: PathBuf,
    },

    #[error("Cannot parse filename, because it contains invalid unicode")]
    InvalidUnicodeError,

    #[error("Cannot parse package size to u32")]
    SizeParseError(#[source] std::num::TryFromIntError),

    #[error("Cannot serialize package metadata")]
    MetadataSerializeError(#[from] toml::ser::Error),

    #[error("Failed to finish creating tar file")]
    TarFinishError(#[source] std::io::Error),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),
}

pub type Result<T> = core::result::Result<T, PackagerError>;

/// Packages a package to a given destination. If the destination doesn't exist, a `PackagerError::InvalidDestination` error is returned.
/// The revision is used to create a unique filename for different package revisions.
pub fn package(config: &Config, package_id: &PackageId, destination: &Path, revisions: u64) -> Result<()> {
    let install_directory = config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

    // Return an error if the destination is not a directory
    if !destination.exists() || !destination.is_dir() {
        return Err(PackagerError::InvalidDestination {
            path: destination.to_path_buf(),
        });
    }

    // Compress the package
    let compressed = compress(&install_directory)?;

    // Calculate checksum and size for the compressed package
    let checksum = Checksum::from_bytes(&compressed);
    let size = FileSize(compressed.len().try_into().map_err(PackagerError::SizeParseError)?);

    let metadata = PrebuildFileMeta { checksum, size };
    let metadata_string = toml::ser::to_string(&metadata)?;

    // Create the file names
    let target_architecture = TargetArchitecture::current().to_string();
    let filename = format!("{package_id}-{revisions}-{target_architecture}");
    let compressed_filename = format!("{filename}.tar.gz");
    let prepackage_file = destination.join(compressed_filename);
    let metadata_filename = format!("{filename}.toml");
    let metadata_file_path = destination.join(metadata_filename);

    // Store the compressed package and metadata
    let mut compressed_file = File::create(&prepackage_file).err_with_path("create", &prepackage_file)?;
    let mut metadata_file = File::create(&metadata_file_path).err_with_path("create", &metadata_file_path)?;
    compressed_file.write_all(&compressed).err_with_path("write", &prepackage_file)?;
    metadata_file.write_all(metadata_string.as_bytes()).err_with_path("write", &metadata_file_path)?;

    Ok(())
}

/// Compresses a given directory using a normalized tar and returns the compressed bytes.
pub fn compress(source_directory: &PathBuf) -> Result<Vec<u8>> {
    let buffer = Vec::new();
    let encoder = GzBuilder::new().mtime(0).write(buffer, Compression::default());

    // Add the whole directory recursively
    let mut tar_builder = Builder::new(encoder);
    create_normalized_tar(&mut tar_builder, &PathBuf::from("."), source_directory)?;
    tar_builder.finish().map_err(PackagerError::TarFinishError)?;

    // Build tar into bytes vec
    let encoder = tar_builder.into_inner().map_err(PackagerError::TarFinishError)?;
    let encoded = encoder.finish().map_err(PackagerError::TarFinishError)?;

    Ok(encoded)
}

/// Creates a normalized tar by recursively adding files to the tar from a given directory while maintaining the directory structure.
fn create_normalized_tar(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    add_directory(builder, tar_path, file_path)?;

    // Get directory entries
    let mut entries = Vec::new();
    for entry in fs::read_dir(file_path).err_with_path("read", file_path)? {
        let entry = entry.err_with_path("iterate", file_path)?;
        entries.push(entry.path());
    }

    // Sort the entries for deterministic behaviour
    entries.sort();

    // Add to tar file recursively
    for entry in entries {
        let filename = entry.file_name().expect("Expected a valid path termination");
        let filename = filename.to_str().ok_or(PackagerError::InvalidUnicodeError)?;

        // Add symlink to tar
        if entry.is_symlink() {
            add_symlink(builder, &tar_path.join(filename), &entry)?;
            continue;
        }

        // Add directory to tar
        if entry.is_dir() {
            create_normalized_tar(builder, &tar_path.join(filename), &entry)?;
            continue;
        }

        // Add file to tar
        if entry.is_file() {
            add_file(builder, &tar_path.join(filename), &entry)?;
            continue;
        }
    }

    Ok(())
}

/// Adds a normalized directory to a tar file.
fn add_directory(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    // Create directory header
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Directory);
    normalize_header(&mut header, 0, tar_path, file_path)?;

    // Add directory to builder
    builder.append_data(&mut header, tar_path, io::empty()).err_with_path("append dir to tar", file_path)?;

    Ok(())
}

/// Adds a normalized file to a tar file.
fn add_file(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    let file = File::open(file_path).err_with_path("open", file_path)?;
    let metadata = file.metadata().err_with_path("read metadata of", file_path)?;

    // Create regular file header
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Regular);
    normalize_header(&mut header, metadata.len(), tar_path, file_path)?;

    // Add file to builder
    builder.append_data(&mut header, tar_path, file).err_with_path("append file to tar", file_path)?;

    Ok(())
}

/// Adds a normalized symlink to a tar file.
fn add_symlink(builder: &mut Builder<GzEncoder<Vec<u8>>>, tar_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    let target = fs::read_link(file_path).err_with_path("read link", file_path)?;

    // Create symlink header
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Symlink);
    normalize_header(&mut header, 0, tar_path, file_path)?;

    // Add symlink to builder
    builder.append_link(&mut header, tar_path, target).err_with_path("append symlink to tar", file_path)?;

    Ok(())
}

/// Normalizes a tar header. Most fields are set to zero. The data length and path fields are set based on
/// the given parameters. The mode field is set based on the entry type of the existing/given header.
fn normalize_header(header: &mut Header, data_length: u64, tar_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    #[cfg(target_os = "windows")]
    let _ = file_path; // Ignore file_path on Windows

    #[cfg(target_family = "unix")]
    {
        // For symlink do symlink_metadata() instead of metadata()
        let metadata = match header.entry_type() == EntryType::Symlink {
            true => fs::symlink_metadata(file_path).err_with_path("read symlink metadata of", file_path)?,
            false => fs::metadata(file_path).err_with_path("read metadata of", file_path)?,
        };
        let mode = metadata.permissions().mode();
        header.set_mode(mode);
    }

    header.set_size(data_length);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_path(tar_path).err_std()?;

    // Reset header checksum
    header.set_cksum();

    Ok(())
}
