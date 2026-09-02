// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::Path};

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

#[derive(Error, Debug)]
pub enum LocalMetadataError {
    #[error("Cannot find metadata file '{file_path}'")]
    MetadataFileNotFound {
        file_path: String,
    },

    #[error("Cannot fetch package metadata from repository")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),

    #[error("Cannot serialize local metadata file")]
    SerializeError(#[from] toml::ser::Error),
}

type Result<T> = core::result::Result<T, LocalMetadataError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalMetadata {
    dependencies: Vec<Dependency>,
}

pub fn store_metadata(provider: &Box<dyn MetadataProvider>, package_id: &PackageId, package_dir: &Path) -> Result<()> {
    let metadata_dir = package_dir.join(DIRECTORY_NAME);

    let package_meta = provider.read_package(&package_id.name)?;
    let package_version_meta = provider.read_package_version(&package_id.name, &package_id.version)?;
    let target_bounds = package_version_meta.get_best_target(&Target::current())?;
    let target_meta = package_version_meta.get_target(&target_bounds)?;

    let local_metadata = create_local_metadata(&package_meta, &package_version_meta, target_meta)?;

    // Create metadata dir if it does not exist
    if !metadata_dir.exists() {
        fs::create_dir_all(&metadata_dir).err_with_path("create dirs", &metadata_dir)?;
    }

    // Write local metadata file
    let local_meta_destination = metadata_dir.join("metadata.toml");
    let local_meta_str = toml::ser::to_string(&local_metadata)?;
    fs::write(&local_meta_destination, local_meta_str).err_with_path("write", local_meta_destination)?;

    // Download external test files
    let external_test_files = package_version_meta.external_test_files.iter().chain(target_meta.external_test_files.iter());
    for external_file in external_test_files {
        download_file(provider, &package_id, external_file, &metadata_dir.join(external_file), true)?;
    }

    // Download test script
    let test_script_path = package_version_meta.get_test_script_path(&target_bounds)?;
    let test_script_destination = metadata_dir.join(format!("test.{SCRIPT_EXTENSION}"));
    download_file(provider, &package_id, &test_script_path, &test_script_destination, false)?;

    // Download uninstall script
    if target_meta.use_uninstall.unwrap_or(package_version_meta.use_uninstall.unwrap_or(false)) {
        let uninstall_script_path = package_version_meta.get_test_script_path(&target_bounds)?;
        let uninstall_script_destination = metadata_dir.join(format!("uninstall.{SCRIPT_EXTENSION}"));
        download_file(provider, &package_id, &uninstall_script_path, &uninstall_script_destination, false)?;
    }

    Ok(())
}

fn create_local_metadata(
    package_meta: &PackageMeta,
    package_version_meta: &PackageVersionMeta,
    target_meta: &PackageTarget,
) -> Result<LocalMetadata> {
    Ok(LocalMetadata {
        dependencies: package_version_meta.dependencies.iter().chain(target_meta.dependencies.iter()).cloned().collect(),
    })
}

fn download_file(
    provider: &Box<dyn MetadataProvider>,
    package_id: &PackageId,
    file_path: &str,
    destination: &Path,
    required: bool,
) -> Result<()> {
    let Some(bytes) = provider.read_file_bytes(&package_id.name, &file_path)? else {
        if !required {
            return Ok(());
        }

        return Err(LocalMetadataError::MetadataFileNotFound {
            file_path: file_path.into(),
        });
    };

    fs::write(destination, bytes).err_with_path("write", destination)?;
    Ok(())
}
