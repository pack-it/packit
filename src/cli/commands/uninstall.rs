// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use std::{collections::VecDeque, process::exit};

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, not_found, standard_print, styled::Styled},
        parameter_checks::{self, contains_package_id},
    },
    config::Config,
    installer::{
        Installer, InstallerOptions,
        types::{OptionalPackageId, PackageId},
    },
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
    packages: Vec<OptionalPackageId>,
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

        // Determine the order in which to uninstall the given packages
        let uninstall_order = self.get_uninstall_order(&register);

        let mut installer = Installer::new(&config, &mut register, &manager, InstallerOptions::default());

        // Uninstall all specified packages
        for optional_id in &uninstall_order {
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

impl UninstallArgs {
    /// Determines the order in which packages need to be uninstalled.
    /// Shows an error and exists if a package is a dependency and the dependent is not in the given packages.
    fn get_uninstall_order(&self, register: &PackageRegister) -> Vec<OptionalPackageId> {
        let mut order = Vec::new();
        let mut packages = VecDeque::from(self.packages.clone());

        while let Some(package) = packages.pop_front() {
            let dependents = self.get_dependents(register, &package);

            // Get all the dependents which are missing in the ordered list
            let mut missing_dependents = Vec::new();
            for dependent in &dependents {
                if !contains_package_id(&order, dependent) {
                    missing_dependents.push(dependent);
                }
            }

            // Add to order if all of its dependents are already in the ordered list
            if missing_dependents.is_empty() {
                order.push(package);
                continue;
            }

            for dependent in &missing_dependents {
                // If none of the given packages covers the dependent throw an error and exit
                if !contains_package_id(&self.packages, &dependent) {
                    error!(
                        msg: "{} cannot be uninstalled, because it is a dependency of the following packages:",
                        package.style()
                    );
                    standard_print::print_list(dependents.into_iter().map(|d| d.style()));
                    exit(1);
                }
            }

            // Note that because we know that the dependent is in the list of given packages it will be before the current package
            // if we add it to the back of the queue.
            packages.push_back(package);
        }

        order
    }

    /// Gets the dependents of the given `optional_id`.
    /// It also checks if the packages exists.
    fn get_dependents(&self, register: &PackageRegister, optional_id: &OptionalPackageId) -> Vec<PackageId> {
        match optional_id.versioned() {
            Some(package_id) if let Some(package_version) = register.get_package_version(&package_id) => {
                return package_version.dependents.iter().cloned().collect();
            },
            Some(package_id) => not_found::register_package_version(&package_id, &register),
            None if let Some(package) = register.get_package(&optional_id.name) => {
                return package.get_versions().iter().flat_map(|p| p.dependents.iter().cloned()).collect();
            },
            None => not_found::register_package(&optional_id.name, &register),
        }
    }
}
