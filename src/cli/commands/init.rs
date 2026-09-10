// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet},
    path::{self, Path, PathBuf},
    process::exit,
    str::FromStr,
};

use chrono::DateTime;
use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::logging::{error, warning},
    },
    config::{Config, EditableConfig, Repository},
    installer::{
        Symlinker,
        types::{PackageId, PackageName, Version},
    },
    platforms::{DEFAULT_CONFIG_DIR, DEFAULT_PREFIX, permissions},
    register::{
        installed_package_version::InstalledPackageVersion,
        metadata::{LocalMetaHandler, LocalMetaPackageHandler, LocalMetadata},
        package_register::PackageRegister,
    },
    repositories::{
        metadata::MetadataProvider,
        types::{LicenseIdentifier, Licenses},
    },
    utils::{
        constants::{DEFAULT_METADATA_REPOSITORY_PROVIDER, DEFAULT_METADATA_REPOSITORY_URL},
        packit_version::packit_version,
        unwrap_or_exit::UnwrapOrExit,
    },
};

/// Initializes the Packit installation.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// The prefix to use
    #[arg(long)]
    prefix: Option<PathBuf>,

    /// The revision of the Packit install
    #[arg(long)]
    revision: Option<u64>,

    /// The descriptions of the revisions to use as fallback
    #[arg(long, num_args = 1..)]
    revision_descriptions: Vec<String>,
}

#[cfg(unix)]
const PACKIT_BINARY_NAME: &str = "packit";

#[cfg(windows)]
const PACKIT_BINARY_NAME: &str = "packit.exe";

impl HandleCommand for InitArgs {
    fn handle(&self) {
        // Check if enough fallback revision descriptions are given (if they are given)
        // Note that this does not require descriptions. If no revisions are given, a fallback value is used.
        if !self.revision_descriptions.is_empty() && self.revision_descriptions.len() as u64 != self.revision.unwrap_or(0) {
            error!(msg: "The number of revision descriptions does not match the given revision");
            exit(1);
        }

        // Check if config directory exists
        let config_dir = Path::new(DEFAULT_CONFIG_DIR);
        if !config_dir.exists() {
            error!(msg: "Packit cannot be initialized: the config directory at '{DEFAULT_CONFIG_DIR}' does not exist yet, please create it first");
            exit(1);
        }

        // Check if config directory is writable
        if !permissions::is_writable(config_dir).unwrap_or_exit_msg("Unable to check if config directory is writable", 1) {
            error!(msg: "Packit cannot be initialized: the config directory at '{DEFAULT_CONFIG_DIR}' is not writable, please set the correct permissions");
            exit(1);
        }

        // Check if config already exists
        if Config::get_default_path().exists() {
            error!(msg: "Packit is already initialized: config already exists");
            exit(2);
        }

        let prefix_directory = match &self.prefix {
            Some(prefix) => path::absolute(prefix).unwrap_or_exit_msg("Unable to convert given prefix to an absolute path", 1),
            None => DEFAULT_PREFIX.into(),
        };

        // Check if prefix directory exists
        if !prefix_directory.exists() {
            error!(msg: "Packit cannot be initialized: the prefix directory at '{}' does not exist yet, please create it first", prefix_directory.display());
            exit(1);
        }

        // Check if prefix directory is writable
        if !permissions::is_writable(&prefix_directory).unwrap_or_exit_msg("Unable to check if prefix directory is writable", 1) {
            error!(msg: "Packit cannot be initialized: the prefix directory at '{}' is not writable, please set the correct permissions", prefix_directory.display());
            exit(1);
        }

        // Check if register already exists
        if PackageRegister::get_path(&prefix_directory).exists() {
            error!(msg: "Packit is already initialized: register already exists");
            exit(2);
        }

        // Check if packit binary is at the correct location
        let packit_package_path = prefix_directory.join("packages").join("packit").join(packit_version!());
        let packit_binary = packit_package_path.join("bin").join(PACKIT_BINARY_NAME);
        if !packit_binary.exists() {
            error!(msg: "Packit cannot be initialized: expected packit binary at '{}'", packit_binary.display());
            exit(1);
        }

        // Create default config
        let mut default_config = EditableConfig::default();
        if let Some(prefix_directory) = &self.prefix {
            default_config.set_prefix_directory(prefix_directory.clone());
        }
        default_config
            .save_to(&Config::get_default_path())
            .unwrap_or_exit_msg("Packit cannot be initialized: error while saving config", 1);

        // Create register containing packit
        let mut register = PackageRegister::new_empty();
        let package_name = PackageName::from_str("packit").expect("Expected 'packit' to be a valid package name");
        let package_version = Version::from_str(packit_version!()).expect("Expected Packit version to be in the correct format");
        let package_id = PackageId::new(package_name, package_version);

        let installed_package_version = InstalledPackageVersion {
            package_id: package_id.clone(),
            revision: self.revision.unwrap_or(0),
            metadata_repository_provider: DEFAULT_METADATA_REPOSITORY_PROVIDER.into(),
            metadata_repository_url: DEFAULT_METADATA_REPOSITORY_URL.into(),
            prebuilds_repository_url: None,
            prebuilds_repository_provider: None,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            install_path: packit_package_path,
            last_metadata_refresh: DateTime::default(), // Initialize to UNIX epoch
            last_metadata_change: DateTime::default(),  // Initialize to UNIX epoch
        };
        let active = false;
        let symlinked = false;
        let package_description =
            "The universal package manager, designed to streamline the experience of installing packages on your system".into();
        let package_homepage = Some("https://github.com/pack-it/packit".into());

        // Add Packit to register
        register.add_package_raw(installed_package_version, active, symlinked, package_description, package_homepage);

        // Save register
        register
            .save_to(&PackageRegister::get_path(&prefix_directory))
            .unwrap_or_exit_msg("Packit cannot be initialized: error while saving register", 1);

        // Create symlinks in prefix directory
        let symlinker = Symlinker::new(default_config.get_config());
        symlinker
            .set_active(&mut register, &package_id, true)
            .unwrap_or_exit_msg("Packit cannot be initialized: error while creating symlinks", 1);

        // Set correct permissions to all files in the prefix
        permissions::set_packit_permissions(&prefix_directory, default_config.get_config().multiuser, true).unwrap_or_exit_msg(
            "Packit cannot be initialized: error while setting permissions of files in the prefix",
            1,
        );

        // Get the installed package from the register
        let Some(installed_package_version) = register.get_package_version_mut(&package_id) else {
            error!(msg: "Packit cannot be initialized: newly created register does not contain the Packit version");
            exit(1);
        };

        // Write local metadata and update register
        let updated = self.write_local_metadata(installed_package_version, &prefix_directory, &package_id);
        installed_package_version.update_metadata_refresh(updated);

        // Save register
        register
            .save_to(&PackageRegister::get_path(&prefix_directory))
            .unwrap_or_exit_msg("Packit cannot be initialized: error while saving register", 1);
    }
}

impl InitArgs {
    /// Writes the local metadata of Packit.
    /// If no remote metadata can be fetched, a fallback is used.
    fn write_local_metadata(
        &self,
        installed_package_version: &InstalledPackageVersion,
        prefix_directory: &Path,
        package_id: &PackageId,
    ) -> bool {
        let local_meta_handler = LocalMetaHandler::new(prefix_directory).get_package(package_id);

        // Create the repository provider to fetch Packit metadata
        let repository = Repository::new(
            &installed_package_version.metadata_repository_url,
            &installed_package_version.metadata_repository_provider,
        );
        let Some(provider) = MetadataProvider::create_from_repository(&repository) else {
            warning!("Using fallback local metadata: metadata provider cannot be created");
            self.write_fallback_local_metadata(local_meta_handler, installed_package_version.revision);
            return true;
        };

        // Fetch Packit metadata from the default repository
        match local_meta_handler.refresh(&provider, installed_package_version.revision) {
            Ok(updated) => updated,
            Err(e) => {
                warning!("Using fallback local metadata: {e}");
                self.write_fallback_local_metadata(local_meta_handler, installed_package_version.revision);
                true
            },
        }
    }

    /// Writes fallback local metadata of Packit.
    fn write_fallback_local_metadata(&self, local_meta_handler: LocalMetaPackageHandler, revision: u64) {
        let revisions = match self.revision_descriptions.is_empty() {
            true => (0..revision).map(|x| format!("Revision {x}")).collect(),
            false => self.revision_descriptions.clone(),
        };

        let local_meta = LocalMetadata {
            required_packit_version: None,
            license: Licenses::Single(LicenseIdentifier("GPL-3.0-only".into())),
            dependencies: Vec::new(),
            test_requirements: Vec::new(),
            external_test_files: HashSet::new(),
            script_args: HashMap::new(),
            deprecation: None,
            skip_symlinking: false,
            conflicts_with: HashSet::new(),
            revisions,
            prebuild: None,
        };

        local_meta_handler
            .write_raw_metadata(local_meta)
            .unwrap_or_exit_msg("Packit cannot be initialized: error while saving fallback local metadata", 1);
    }
}
