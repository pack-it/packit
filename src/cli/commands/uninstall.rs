// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use std::process::exit;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, not_found, standard_print, styled::Styled},
        parameter_checks,
    },
    config::Config,
    installer::{Installer, InstallerOptions, types::OptionalPackageId},
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Uninstalls the specified packages, if a version is given that version will be uninstalled, if not,
/// you will be asked if you want to delete all versions of <PACKAGE-NAME> in case there are multiple
/// versions installed. Multiple packages can be specified by entering multiple names, split by a space.
#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// The names of the packages to install, with an optional version specified with `<name>[@version]`
    #[arg(required = true)]
    pub packages: Vec<OptionalPackageId>,
}

impl HandleCommand for UninstallArgs {
    fn handle(&self) {
        // Check for duplicates, because uninstalling twice will result in a confusing error
        let duplicates = parameter_checks::get_duplicates(&self.packages);
        if !duplicates.is_empty() {
            error!(msg: "Duplicate package arguments are not allowed. The following duplicates were found:");
            standard_print::print_list(duplicates.iter());
            exit(1);
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Check if all packages are installed before starting uninstall
        for optional_id in &self.packages {
            match optional_id.versioned() {
                Some(package_id) if register.get_package_version(&package_id).is_some() => continue,
                Some(package_id) => not_found::register_package_version(&package_id, &register),
                None if register.get_package(&optional_id.name).is_some() => continue,
                None => not_found::register_package(&optional_id.name, &register),
            }
        }

        let mut installer = Installer::new(&config, &mut register, &manager, InstallerOptions::default());

        // Uninstall all specified packages
        for optional_id in &self.packages {
            match installer.uninstall(optional_id) {
                Ok(uninstalled_packages) => {
                    for package in uninstalled_packages {
                        let styled_message = format!("Successfully uninstalled {}", package.style()).bold().green();
                        println!("{styled_message}");
                    }
                },
                Err(error) => error!(error, "Cannot uninstall package {}", optional_id.style()),
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
