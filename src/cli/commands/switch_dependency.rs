use std::process::exit;

// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{logging::error, not_found, styled::Styled},
    },
    config::Config,
    installer::{
        Symlinker,
        types::{PackageId, PackageName, Version},
    },
    register::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Switches a dependency version of the specified package to the specified version.
#[derive(Args, Debug)]
pub struct SwitchDependencyArgs {
    /// The name of the package to switch dependency of
    pub package: PackageId,

    /// The name of the dependency to switch
    pub dependency_name: PackageName,

    /// The new version of the dependency
    pub dependency_version: Version,
}

impl HandleCommand for SwitchDependencyArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_path = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_path).unwrap_or_exit(1);

        // Get installed package
        let package = match register.get_package_version(&self.package) {
            Some(package) => package,
            None => not_found::register_package_version(&self.package, &register),
        };

        // Check if dependency exists
        let current_version = package.dependencies.iter().find(|x| x.name == self.dependency_name);
        let Some(current_version) = current_version else {
            error!(msg: "Package {} does not have {} as dependency", self.package.style(), self.dependency_name.style());
            exit(1);
        };

        // Check if dependency is already at the new version
        if current_version.version == self.dependency_version {
            println!(
                "{} already uses dependency {} version {}",
                self.package.style(),
                self.dependency_name.style(),
                self.dependency_version.style()
            );
            return;
        }

        // Get dependency
        let dependency = match register.get_package(&self.dependency_name) {
            Some(package) => package,
            None => {
                error!(msg: "Dependency {} is not installed, this is unexpected behaviour", self.dependency_name.style());
                exit(1)
            },
        };

        // Check if new version is installed
        if !dependency.versions.keys().any(|x| *x == self.dependency_version) {
            error!(msg: "Dependency version {} is not installed", self.dependency_version.style());
            not_found::display_versions(dependency.versions.keys());
        }

        // Switch dependency using symlinker
        let current_version = current_version.clone();
        let symlinker = Symlinker::new(&config);
        if let Err(e) = symlinker.switch_dependency(&mut register, &self.package, &current_version, self.dependency_version.clone()) {
            error!(e, "Error while trying to switch dependency");
            exit(1);
        }

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);

        let styled_message = format!(
            "Successfully switched dependency {} of {} to {}",
            self.dependency_name.style(),
            self.package.style(),
            self.dependency_version.style()
        );

        println!("{}", styled_message.bold().green());
    }
}
