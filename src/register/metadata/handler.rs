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
    register::{
        installed_package::InstalledPackage,
        metadata::error::{LocalMetadataError, Result},
        package_register::PackageRegister,
    },
    repositories::{
        metadata::MetadataProvider,
        types::{DeprecationInfo, Licenses, PackageMeta, PackageVersionMeta, PrebuildMeta, PrebuildsList, Requirement, TargetBounds},
    },
    utils::{ioerror::IOResultExt, serialization},
};

const METADATA_FILENAME: &str = "metadata.toml";

/// Represents the local metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalMetadata {
    pub required_packit_version: Option<Version>,

    #[serde(default, skip_serializing_if = "Licenses::is_unknown")]
    pub license: Licenses,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_requirements: Vec<Requirement>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    #[serde(serialize_with = "serialization::serialize_set_sorted")]
    pub external_test_files: HashSet<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[serde(serialize_with = "serialization::serialize_map_sorted")]
    pub script_args: HashMap<String, String>,

    pub deprecation: Option<DeprecationInfo>,

    #[serde(default = "PackageVersionMeta::default_skip_symlinking")]
    #[serde(skip_serializing_if = "PackageVersionMeta::is_default_skip_symlinking")]
    pub skip_symlinking: bool,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    #[serde(serialize_with = "serialization::serialize_set_sorted")]
    pub conflicts_with: HashSet<PackageName>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<String>,

    pub prebuild: Option<LocalPrebuildMetadata>,
}

/// Represents a prebuild in local metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalPrebuildMetadata {
    pub id: String,

    #[serde(flatten)]
    pub info: PrebuildMeta,
}

/// Handler which handles the reading and refreshing of local metadata.
pub struct LocalMetaHandler<'a> {
    prefix_dir: &'a Path,
}

/// Handler which handles the reading and refreshing of local metadata for a specific package.
pub struct LocalMetaPackageHandler<'a> {
    prefix_dir: &'a Path,
    package_id: &'a PackageId,
}

impl<'a> LocalMetaHandler<'a> {
    /// Creates a new `LocalMetaHandler`.
    pub fn new(prefix_dir: &'a Path) -> Self {
        Self { prefix_dir }
    }

    /// Gets the local metadata handler for the given package.
    pub fn get_package(&self, package_id: &'a PackageId) -> LocalMetaPackageHandler<'a> {
        LocalMetaPackageHandler {
            prefix_dir: self.prefix_dir,
            package_id,
        }
    }

    /// Gets the conflicts of the given package.
    /// Returns a list of conflicts defined in the metadata, which can contain packages that are not installed.
    pub fn read_package_conflicts(&self, package: &InstalledPackage) -> Result<HashSet<PackageName>> {
        let mut conflicts = HashSet::new();

        for version in package.versions.values() {
            let metadata = self.get_package(&version.package_id).read_metadata()?;
            conflicts.extend(metadata.conflicts_with);
        }

        Ok(conflicts)
    }

    /// Checks if a package has conflicts with any package that is installed.
    /// Returns a list of the names of all conflicting packages.
    pub fn get_conflicting_packages(
        &self,
        register: &PackageRegister,
        package_name: &PackageName,
        package_conflicts: &HashSet<PackageName>,
    ) -> Result<HashSet<PackageName>> {
        let mut conflicting_packages = HashSet::new();

        for (name, package) in register.iterate_packages() {
            if !package.symlinked || name == package_name {
                continue;
            }

            // Check if the package specifies this package as conflict
            if package_conflicts.contains(name) {
                conflicting_packages.insert(name.clone());
                continue;
            }

            // Check if this package specifies the package as conflict
            let conflicts = self.read_package_conflicts(package)?;
            if conflicts.contains(package_name) {
                conflicting_packages.insert(name.clone());
            }
        }

        Ok(conflicting_packages)
    }
}

impl<'a> LocalMetaPackageHandler<'a> {
    /// Gets the base path of the local metadata storage for the current package.
    fn get_base_path(&self) -> PathBuf {
        self.prefix_dir.join("metadata").join(&self.package_id.name).join(self.package_id.version.to_string())
    }

    /// Reads the local metadata file from the storage of the given package.
    /// Returns the `LocalMetadata` parsed from the storage.
    pub fn read_metadata(&self) -> Result<LocalMetadata> {
        let path = self.get_base_path().join(METADATA_FILENAME);
        if !path.exists() {
            return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
        }

        let content = fs::read_to_string(&path).err_with_path("read", &path)?;
        Ok(toml::de::from_str(&content)?)
    }

    /// Reads the specified local metadata file from the storage of the given package.
    /// Returns the file as string.
    pub fn read_file(&self, file: &str) -> Result<String> {
        let path = self.get_base_path().join(file);
        if !path.exists() {
            return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
        }

        let content = fs::read_to_string(&path).err_with_path("read", &path)?;
        Ok(content)
    }

    /// Reads the specified local metadata file from the storage of the given package.
    /// Returns the file as bytes.
    pub fn read_file_bytes(&self, file: &str) -> Result<Bytes> {
        let path = self.get_base_path().join(file);
        if !path.exists() {
            return Err(LocalMetadataError::LocalMetadataFileNotFound { file_path: path });
        }

        let content = fs::read(&path).err_with_path("read", &path)?;
        Ok(content.into())
    }

    /// Reads the test script from the storage of the given package.
    /// Returns the file as string.
    pub fn read_test_script(&self) -> Result<String> {
        self.read_file(&format!("test.{SCRIPT_EXTENSION}"))
    }

    /// Reads the uninstall script from the storage of the given package.
    /// Returns the file as string.
    pub fn read_uninstall_script(&self) -> Result<String> {
        self.read_file(&format!("uninstall.{SCRIPT_EXTENSION}"))
    }

    /// Refreshes the local metadata of the given package.
    /// Returns true if the metadata was changed, false otherwise.
    pub fn refresh(&self, provider: &MetadataProvider, current_revision: u64) -> Result<bool> {
        let metadata_dir = self.get_base_path();

        let package_meta = provider.read_package(&self.package_id.name)?;
        let package_version_meta = provider.read_package_version(&self.package_id.name, &self.package_id.version)?;
        let target_bounds = package_version_meta.get_best_target(&Target::current())?;
        let target_meta = package_version_meta.get_target(&target_bounds)?;
        let prebuilds_list = provider.read_prebuilds_list(&self.package_id.name, &self.package_id.version)?;

        // Check if current revision is the same as the revision of the metadata
        if package_version_meta.get_revision_count() != current_revision {
            return Err(LocalMetadataError::MetadataRevisionMismatch);
        }

        let local_metadata = self.create_local_metadata(&package_meta, &package_version_meta, &target_bounds, prebuilds_list)?;
        let local_meta_str = toml::ser::to_string(&local_metadata)?;

        let mut updated = false;

        // Create metadata dir if it does not exist
        if !metadata_dir.exists() {
            fs::create_dir_all(&metadata_dir).err_with_path("create dirs", &metadata_dir)?;
            updated = true;
        }

        // Collect a list of all files in the metadata directory before refreshing
        // Note that this assumes a flat metadata directory structure
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
            let normalized_file_name = normalize_external_test_file_path(external_file);

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
            let uninstall_script_path = package_version_meta.get_uninstall_script_path(&target_bounds)?;
            let uninstall_script_destination = metadata_dir.join(format!("uninstall.{SCRIPT_EXTENSION}"));
            let new_file = self.request_file(provider, &uninstall_script_path, true)?;
            if self.write_file_if_changed(&before_files, &mut after_files, uninstall_script_destination, new_file)? {
                updated = true;
            }
        }

        // Remove files that are not needed anymore
        let removed_files = before_files.iter().filter(|x| !after_files.contains(x));
        for removed_file in removed_files {
            fs::remove_file(removed_file).err_with_path("remove", removed_file)?;
            updated = true;
        }

        Ok(updated)
    }

    /// Writes the given `LocalMetadata` for the package.
    /// Note that this should normally not be used, it only exists for use in the init command.
    pub fn write_raw_metadata(&self, metadata: LocalMetadata) -> Result<()> {
        let metadata_dir = self.get_base_path();

        // Create metadata dir if it does not exist
        if !metadata_dir.exists() {
            fs::create_dir_all(&metadata_dir).err_with_path("create dirs", &metadata_dir)?;
        }

        let metadata_file = metadata_dir.join(METADATA_FILENAME);
        let local_meta_str = toml::ser::to_string(&metadata)?;

        // Write metadata to file
        fs::write(&metadata_file, local_meta_str).err_with_path("write", &metadata_file)?;

        Ok(())
    }

    /// Creates the local metadata from the given package, version and target metadata.
    /// Returns the created `LocalMetadata`.
    fn create_local_metadata(
        &self,
        package_meta: &PackageMeta,
        package_version_meta: &PackageVersionMeta,
        target_bounds: &TargetBounds,
        prebuilds_list: Option<PrebuildsList>,
    ) -> Result<LocalMetadata> {
        let target_meta = package_version_meta.get_target(target_bounds)?;

        let required_packit_version = max(&package_meta.required_packit_version, &package_version_meta.required_packit_version);
        let dependencies = package_version_meta.dependencies.iter().chain(target_meta.dependencies.iter()).cloned().collect();

        // Normalize external test files to flatten directories into the name
        let external_test_files = package_version_meta
            .get_external_test_files(target_bounds)?
            .into_iter()
            .map(normalize_external_test_file_path)
            .collect();

        let script_args =
            package_version_meta.get_script_args(target_bounds)?.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        let deprecation = package_meta.deprecation.as_ref().or(package_version_meta.deprecation.as_ref()).cloned();

        let mut prebuild = None;
        if let Some(prebuilds_list) = prebuilds_list {
            if let Some((prebuild_id, prebuild_meta)) = prebuilds_list.get_best_prebuild(&Target::current()) {
                prebuild = Some(LocalPrebuildMetadata {
                    id: prebuild_id.clone(),
                    info: prebuild_meta.clone(),
                });
            }
        }

        Ok(LocalMetadata {
            required_packit_version: required_packit_version.clone(),
            license: package_version_meta.license.clone(),
            dependencies,
            test_requirements: target_meta.test_requirements.clone(),
            external_test_files,
            script_args,
            deprecation,
            skip_symlinking: target_meta.skip_symlinking.unwrap_or(package_version_meta.skip_symlinking),
            conflicts_with: package_meta.conflicts_with.clone(),
            revisions: package_version_meta.revisions.clone(),
            prebuild,
        })
    }

    /// Requests a file from the given provider.
    /// If the file cannot be found, it returns an `LocalMetadataError::MetadataFileNotFound`, or None if the file is not required.
    /// Returns the bytes of the file if it can be found.
    fn request_file(&self, provider: &MetadataProvider, file_path: &str, required: bool) -> Result<Option<Bytes>> {
        let Some(bytes) = provider.read_file_bytes(&self.package_id.name, file_path)? else {
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
        before_files: &[PathBuf],
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

            // If the file did not change, skip writing and store it in `after_files`
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

/// Normalizes the external test file path by flattening directories into the filename.
fn normalize_external_test_file_path(external_test_file: &str) -> String {
    external_test_file.replace("/", "-")
}
