// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{not_found, styled::Styled},
    },
    config::Config,
    installer::{Symlinker, types::PackageName},
    register::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Unlinks the specified package, causing the package to be unavailable from the PATH environment variable.
#[derive(Args, Debug)]
pub struct UnlinkArgs {
    /// The name of the package to unlink
    pub package_name: PackageName,
}

impl HandleCommand for UnlinkArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_path = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_path).unwrap_or_exit(1);

        // Get installed package
        let package = match register.get_package(&self.package_name) {
            Some(package) => package,
            None => not_found::register_package(&self.package_name, &register),
        };

        // Check if the package is already symlinked
        if !package.symlinked {
            println!("This package is currently not symlinked");
            return;
        }

        // Unlink package
        Symlinker::new(&config)
            .unlink_package(&mut register, &self.package_name)
            .unwrap_or_exit_msg("Unable to unlink package", 1);

        println!("Successfully unlinked {}", self.package_name.style());

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);
    }
}
