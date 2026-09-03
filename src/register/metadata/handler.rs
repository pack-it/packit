// SPDX-License-Identifier: GPL-3.0-only
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
    installer::{
        scripts::SCRIPT_EXTENSION,
        types::{Dependency, PackageId, PackageName, Version},
    },
    platforms::Target,
    register::metadata::error::{LocalMetadataError, Result},
    repositories::{
        provider::MetadataProvider,
        types::{DeprecationInfo, PackageMeta, PackageVersionMeta, Requirement, TargetBounds},
    },
    utils::ioerror::IOResultExt,
};

pub const DIRECTORY_NAME: &str = ".packit";
const METADATA_FILENAME: &str = "metadata.toml";

/// Represents the local metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalMetadata {
    pub required_packit_version: Option<Version>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_requirements: Vec<Requirement>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub external_test_files: HashSet<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub script_args: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<String>,
    pub deprecation: Option<DeprecationInfo>,
    pub skip_symlinking: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts_with: Vec<PackageName>,
}

/// Handler which handles the reading and refreshing of local metadata.
pub struct LocalMetaHandler<'a> {
    package_id: &'a PackageId,
    package_install_dir: &'a Path,
}

impl<'a> LocalMetaHandler<'a> {
    /// Creates a new `LocalMetaHandler` for the given package.
    pub fn new(package_id: &'a PackageId, package_install_dir: &'a Path) -> Self {
        Self {
            package_id,
            package_install_dir,
        }
    }

    /// Reads the local metadata file from the storage of the given package.
    /// Returns the `LocalMetadata` parsed from the storage.
    pub fn read_metadata(&self) -> Result<LocalMetadata> {
        let path = self.package_install_dir.join(DIRECTORY_NAME).join(METADATA_FILENAME);
        if !path.exists() {
            return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
        }

        let content = fs::read_to_string(&path).err_with_path("read", &path)?;
        Ok(toml::de::from_str(&content)?)
    }

    /// Reads the specified local metadata file from the storage of the given package.
    /// Returns the file as string.
    pub fn read_file(&self, file: &str) -> Result<String> {
        let path = self.package_install_dir.join(DIRECTORY_NAME).join(file);
        if !path.exists() {
            return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
        }

        let content = fs::read_to_string(&path).err_with_path("read", &path)?;
        Ok(content)
    }

    /// Reads the specified local metadata file from the storage of the given package.
    /// Returns the file as bytes.
    pub fn read_file_bytes(&self, file: &str) -> Result<Bytes> {
        let path = self.package_install_dir.join(DIRECTORY_NAME).join(file);
        if !path.exists() {
            return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
        }

        let content = fs::read(&path).err_with_path("read", &path)?;
        Ok(content.into())
    }

    /// Refreshes the local metadata of the given package.
    /// Returns true if the metadata was changed, false otherwise.
    pub fn refresh(&self, provider: &Box<dyn MetadataProvider>) -> Result<bool> {
        let metadata_dir = self.package_install_dir.join(DIRECTORY_NAME);

        let package_meta = provider.read_package(&self.package_id.name)?;
        let package_version_meta = provider.read_package_version(&self.package_id.name, &self.package_id.version)?;
        let target_bounds = package_version_meta.get_best_target(&Target::current())?;
        let target_meta = package_version_meta.get_target(&target_bounds)?;

        let local_metadata = self.create_local_metadata(&package_meta, &package_version_meta, &target_bounds)?;
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
        if self.write_file_if_changed(&before_files, &mut after_files, local_meta_destination, Some(local_meta_bytes))? {
            updated = true;
        }

        // Download external test files
        let external_test_files = package_version_meta.get_external_test_files(&target_bounds)?;
        for external_file in external_test_files {
            // Flatten external file directory names
            let normalized_file_name = external_file.replace("/", "-");

            let destination = metadata_dir.join(normalized_file_name);
            let new_file = self.request_file(provider, external_file, true)?;
            if self.write_file_if_changed(&before_files, &mut after_files, destination, new_file)? {
                updated = true;
            }
        }

        // Download test script
        let test_script_path = package_version_meta.get_test_script_path(&target_bounds)?;
        let test_script_destination = metadata_dir.join(format!("test.{SCRIPT_EXTENSION}"));
        let new_file = self.request_file(provider, &test_script_path, false)?;
        if self.write_file_if_changed(&before_files, &mut after_files, test_script_destination, new_file)? {
            updated = true;
        }

        // Download uninstall script
        if target_meta.use_uninstall.unwrap_or(package_version_meta.use_uninstall.unwrap_or(false)) {
            let uninstall_script_path = package_version_meta.get_test_script_path(&target_bounds)?;
            let uninstall_script_destination = metadata_dir.join(format!("uninstall.{SCRIPT_EXTENSION}"));
            let new_file = self.request_file(provider, &uninstall_script_path, true)?;
            if self.write_file_if_changed(&before_files, &mut after_files, uninstall_script_destination, new_file)? {
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
        &self,
        package_meta: &PackageMeta,
        package_version_meta: &PackageVersionMeta,
        target_bounds: &TargetBounds,
    ) -> Result<LocalMetadata> {
        let target_meta = package_version_meta.get_target(target_bounds)?;

        let required_packit_version = max(&package_meta.required_packit_version, &package_version_meta.required_packit_version);
        let dependencies = package_version_meta.dependencies.iter().chain(target_meta.dependencies.iter()).cloned().collect();

        // Normalize external test files to flatten directories into the name
        let external_test_files = package_version_meta
            .get_external_test_files(target_bounds)?
            .iter()
            .map(ToString::to_string)
            .map(|x| x.replace("/", "-"))
            .collect();

        let script_args =
            package_version_meta.get_script_args(target_bounds)?.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        let deprecation = package_meta.deprecation.as_ref().or(package_version_meta.deprecation.as_ref()).cloned();

        Ok(LocalMetadata {
            required_packit_version: required_packit_version.clone(),
            dependencies,
            test_requirements: target_meta.test_requirements.clone(),
            external_test_files,
            script_args,
            revisions: package_version_meta.revisions.clone(),
            deprecation,
            skip_symlinking: target_meta.skip_symlinking.unwrap_or(package_version_meta.skip_symlinking),
            conflicts_with: package_meta.conflicts_with.clone(),
        })
    }

    /// Requests a file from the given provider.
    /// If the file cannot be found, it returns an `LocalMetadataError::MetadataFileNotFound`, or None if the file is not required.
    /// Returns the bytes of the file if it can be found.
    fn request_file(&self, provider: &Box<dyn MetadataProvider>, file_path: &str, required: bool) -> Result<Option<Bytes>> {
        let Some(bytes) = provider.read_file_bytes(&self.package_id.name, &file_path)? else {
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
        &self,
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
}
