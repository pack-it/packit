// SPDX-License-Identifier: GPL-3.0-only
use std::{
    fs,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    installer::{
        scripts::SCRIPT_EXTENSION,
        types::{Dependency, PackageId},
    },
    platforms::Target,
    repositories::{
        error::RepositoryError,
        provider::MetadataProvider,
        types::{PackageMeta, PackageTarget, PackageVersionMeta},
    },
    utils::ioerror::{self, IOResultExt},
};

pub const DIRECTORY_NAME: &str = ".packit";
const METADATA_FILENAME: &str = "metadata.toml";

#[derive(Error, Debug)]
pub enum LocalMetadataError {
    #[error("Cannot find metadata file '{file_path}' in source repository")]
    RepositoryMetadataFileNotFound {
        file_path: String,
    },

    #[error("Cannot find local metadata file '{}'", file_path.display())]
    LocalMetadataFileNotFound {
        file_path: PathBuf,
    },

    #[error("Cannot fetch package metadata from repository")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot parse local metadata file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot serialize local metadata file")]
    SerializeError(#[from] toml::ser::Error),
}

type Result<T> = core::result::Result<T, LocalMetadataError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalMetadata {
    dependencies: Vec<Dependency>,
}

/// Reads the local metadata file from the storage of the given package.
/// Returns the `LocalMetadata` parsed from the storage.
pub fn read_local_metadata(package_install_dir: &Path) -> Result<LocalMetadata> {
    let path = package_install_dir.join(DIRECTORY_NAME).join(METADATA_FILENAME);
    if !path.exists() {
        return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
    }

    let content = fs::read_to_string(&path).err_with_path("read", &path)?;
    Ok(toml::de::from_str(&content)?)
}

/// Reads the specified local metadata file from the storage of the given package.
/// Returns the file as bytes.
pub fn read_local_meta_file(package_install_dir: &Path, file: &str) -> Result<Bytes> {
    let path = package_install_dir.join(DIRECTORY_NAME).join(file);
    if !path.exists() {
        return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
    }

    let content = fs::read(&path).err_with_path("read", &path)?;
    Ok(content.into())
}

/// Refreshes the local metadata of the given package.
/// Returns true if the metadata was changed, false otherwise.
pub fn refresh_metadata(provider: &Box<dyn MetadataProvider>, package_id: &PackageId, package_install_dir: &Path) -> Result<bool> {
    let metadata_dir = package_install_dir.join(DIRECTORY_NAME);

    let package_meta = provider.read_package(&package_id.name)?;
    let package_version_meta = provider.read_package_version(&package_id.name, &package_id.version)?;
    let target_bounds = package_version_meta.get_best_target(&Target::current())?;
    let target_meta = package_version_meta.get_target(&target_bounds)?;

    let local_metadata = create_local_metadata(&package_meta, &package_version_meta, target_meta)?;
    let local_meta_str = toml::ser::to_string(&local_metadata)?;

    let mut updated = false;

    // Create metadata dir if it does not exist
    if !metadata_dir.exists() {
        fs::create_dir_all(&metadata_dir).err_with_path("create dirs", &metadata_dir)?;
        updated = true;
    }

    // Collect a list of all files in the metadata directory before refreshing
    let mut before_files = Vec::new();
    for entry in fs::read_dir(&metadata_dir).err_with_path("read", &metadata_dir)? {
        let entry = entry.err_with_path("iterate", &metadata_dir)?;
        before_files.push(entry.path());
    }
    let mut after_files = Vec::new();

    // If the metadata is updated, write the metadata to the file
    let local_meta_destination = metadata_dir.join(METADATA_FILENAME);
    let local_meta_bytes = local_meta_str.into();
    if write_file_if_changed(&before_files, &mut after_files, local_meta_destination, Some(local_meta_bytes))? {
        updated = true;
    }

    // Download external test files
    let external_test_files = package_version_meta.external_test_files.iter().chain(target_meta.external_test_files.iter());
    for external_file in external_test_files {
        let destination = metadata_dir.join(external_file);
        let new_file = request_file(provider, &package_id, external_file, true)?;
        if write_file_if_changed(&before_files, &mut after_files, destination, new_file)? {
            updated = true;
        }
    }

    // Download test script
    let test_script_path = package_version_meta.get_test_script_path(&target_bounds)?;
    let test_script_destination = metadata_dir.join(format!("test.{SCRIPT_EXTENSION}"));
    let new_file = request_file(provider, &package_id, &test_script_path, false)?;
    if write_file_if_changed(&before_files, &mut after_files, test_script_destination, new_file)? {
        updated = true;
    }

    // Download uninstall script
    if target_meta.use_uninstall.unwrap_or(package_version_meta.use_uninstall.unwrap_or(false)) {
        let uninstall_script_path = package_version_meta.get_test_script_path(&target_bounds)?;
        let uninstall_script_destination = metadata_dir.join(format!("uninstall.{SCRIPT_EXTENSION}"));
        let new_file = request_file(provider, &package_id, &uninstall_script_path, true)?;
        if write_file_if_changed(&before_files, &mut after_files, uninstall_script_destination, new_file)? {
            updated = true;
        }
    }

    // Remove files that are not needed anymore
    let removed_files: Vec<_> = before_files.iter().filter(|x| !after_files.contains(x)).collect();
    for removed_file in removed_files {
        fs::remove_file(removed_file).err_with_path("remove", removed_file)?;
        updated = true;
    }

    Ok(updated)
}

/// Creates the local metadata from the given package, version and target metadata.
/// Returns the created `LocalMetadata`.
fn create_local_metadata(
    package_meta: &PackageMeta,
    package_version_meta: &PackageVersionMeta,
    target_meta: &PackageTarget,
) -> Result<LocalMetadata> {
    Ok(LocalMetadata {
        dependencies: package_version_meta.dependencies.iter().chain(target_meta.dependencies.iter()).cloned().collect(),
    })
}

/// Requests a file from the given provider.
/// If the file cannot be found, it returns an `LocalMetadataError::MetadataFileNotFound`, or None if the file is not required.
/// Returns the bytes of the file if it can be found.
fn request_file(provider: &Box<dyn MetadataProvider>, package_id: &PackageId, file_path: &str, required: bool) -> Result<Option<Bytes>> {
    let Some(bytes) = provider.read_file_bytes(&package_id.name, &file_path)? else {
        if !required {
            return Ok(None);
        }

        return Err(LocalMetadataError::RepositoryMetadataFileNotFound {
            file_path: file_path.into(),
        });
    };

    Ok(Some(bytes))
}

/// Writes a metadata file when it is changed.
/// Compares the new content with the old content of the file.
/// Updates the list `after_files` when a file should be kept.
/// Returns true if the file changed, false otherwise.
fn write_file_if_changed(
    before_files: &Vec<PathBuf>,
    after_files: &mut Vec<PathBuf>,
    destination: PathBuf,
    new_content: Option<Bytes>,
) -> Result<bool> {
    // Get new content, return when the file is not available
    // True is returned when the file was present before, false if the file never existed
    let Some(new_content) = new_content else {
        return Ok(before_files.contains(&destination));
    };

    // If the file already existed, check content equality
    if before_files.contains(&destination) {
        let old_content = fs::read(&destination).err_with_path("read", &destination)?;

        // If the file did not change, skip writing and store it as new file
        if new_content == old_content {
            after_files.push(destination);
            return Ok(false);
        }
    }

    // Write new file data
    fs::write(&destination, new_content).err_with_path("write", &destination)?;
    after_files.push(destination);
    Ok(true)
}
