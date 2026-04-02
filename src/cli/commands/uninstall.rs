// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{Installer, InstallerOptions, types::OptionalPackageId},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::{duplicates, unwrap_or_exit::UnwrapOrExit},
};

/// Uninstalls the specified packages, if a version is given that version will be uninstalled, if not,
/// you will be asked if you want to delete all versions of <PACKAGE-NAME> in case there are multiple
/// versions installed. Multiple packages can be specified by entering multiple names, split by a space.
#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// The names of the packages to install, with an optional version specified with `<name>[@version]`.
    #[arg(required = true)]
    pub packages: Vec<OptionalPackageId>,
}

impl HandleCommand for UninstallArgs {
    fn handle(&self) {
        // Check for duplicates, because uninstalling twice will result in a confusing error
        let duplicates = duplicates::get_duplicates(&self.packages);
        if !duplicates.is_empty() {
            let mut duplicate_string = String::new();
            for duplicate in duplicates {
                duplicate_string.push_str(&duplicate.to_string());
                duplicate_string.push(' ');
            }

            error!(msg: "Duplicate package arguments are not allowed. The following duplicates were found: {duplicate_string}");
            exit(1);
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let mut installer = Installer::new(&config, &mut register, &manager, InstallerOptions::default());

        // Uninstall all specified packages
        for optional_id in &self.packages {
            match installer.uninstall(&optional_id) {
                Ok(uninstalled_packages) => {
                    let mut uninstalled_string = String::new();
                    for package in uninstalled_packages {
                        uninstalled_string.push_str(&package.to_string());
                        uninstalled_string.push(' ');
                    }
                    println!("Successfully uninstalled: {uninstalled_string}");
                },
                Err(error) => error!(error, "Cannot uninstall package {optional_id}"),
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
