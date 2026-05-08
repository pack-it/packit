// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, path::PathBuf, process::exit, str::FromStr};

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::{Config, EditableConfig},
    installer::types::{PackageId, PackageName, Version},
    platforms::DEFAULT_PREFIX,
    repositories::types::Licenses,
    storage::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    utils::{
        constants::{DEFAULT_METADATA_REPOSITORY_PATH, DEFAULT_METADATA_REPOSITORY_PROVIDER},
        unwrap_or_exit::UnwrapOrExit,
    },
};

/// Initializes the Packit installation.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// The prefix to use
    prefix: Option<PathBuf>,
}

impl HandleCommand for InitArgs {
    fn handle(&self) {
        // Check if config already exists
        if Config::get_default_path().exists() {
            error!(msg: "Packit is already initialized: config already exists");
            exit(2);
        }

        let prefix_directory = match &self.prefix {
            Some(prefix) => prefix.clone(),
            None => DEFAULT_PREFIX.into(),
        };

        // Check if register already exists
        if PackageRegister::get_default_path(&prefix_directory).exists() {
            error!(msg: "Packit is already initialized: register already exists");
            exit(2);
        }

        let packit_version = env!("CARGO_PKG_VERSION");

        // Check if packit binary is at the correct location
        let packit_binary = prefix_directory.join("packages").join("packit").join(packit_version).join("bin").join("packit");
        if !packit_binary.exists() {
            error!(msg: "Packit cannot be initialized: expected packit binary at {}", packit_binary.display());
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
        let package_version = Version::from_str(packit_version).expect("Expected Packit version to be in the correct format");

        let installed_package_version = InstalledPackageVersion {
            package_id: PackageId::new(package_name, package_version),
            license: Licenses::Single("GPL-3.0-only".into()),
            source_repository_provider: DEFAULT_METADATA_REPOSITORY_PROVIDER.into(),
            source_repository_url: DEFAULT_METADATA_REPOSITORY_PATH.into(),
            source_prebuild_repository_url: None,
            source_prebuild_repository_provider: None,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            install_path: packit_binary,
            revisions: Vec::new(),
        };
        let active = false; //TODO
        let symlinked = false; //TODO
        let package_description =
            "The universal package manager, designed to streamline the experience of installing packages on your system.".into();
        let package_homepage = Some("https://github.com/pack-it/packit".into());

        // Add Packit to register
        register
            .add_package_raw(installed_package_version, active, symlinked, package_description, package_homepage)
            .unwrap_or_exit_msg("Packit cannot be initialized: error while adding Packit to register", 1);

        // Save register
        register
            .save_to(&PackageRegister::get_default_path(&prefix_directory))
            .unwrap_or_exit_msg("Packit cannot be initialized: error while saving register", 1);
    }
}
