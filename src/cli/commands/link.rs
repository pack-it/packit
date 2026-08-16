// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;
use colored::Colorize;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{
            logging::{error, warning},
            not_found, standard_print,
            styled::{MapStyled, Styled},
        },
    },
    config::{Config, Repository},
    installer::{Symlinker, types::PackageName},
    platforms::Target,
    register::{
        installed_package::InstalledPackage, installed_package_version::InstalledPackageVersion, package_register::PackageRegister,
    },
    repositories::provider,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Links the specified package into the /bin, /lib, /share, etc. directories.
#[derive(Args, Debug)]
pub struct LinkArgs {
    /// The name of the package to link
    package_name: PackageName,

    /// True to force linking, even when we should not link
    #[arg(short, long, default_value = "false")]
    force: bool,

    /// True to overwrite existing links from another package, false to skip existing links
    #[arg(long, default_value = "false")]
    overwrite: bool,
}

impl HandleCommand for LinkArgs {
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
        if !self.overwrite && package.symlinked {
            println!("This package is already symlinked");
            return;
        }

        // Show warning if forced
        if self.force {
            warning!("Forcing symlink can cause problems, please be carefull when using '--force'");
        }

        // Get active package version
        let package_version = package
            .get_package_version(&package.active_version)
            .unwrap_or_exit_msg("Unable to retrieve active version of package", 1);

        // Check if linking is allowed, exit if force is not enabled
        if !self.linking_allowed(&register, package, package_version) {
            if !self.force {
                println!("Try '--force' if you are sure you want to link, note that this can result in issues");
                exit(1);
            }
        }

        // Create symlinks
        Symlinker::new(&config)
            .create_symlinks(&package_version.install_path, self.overwrite)
            .unwrap_or_exit_msg("Unable to link package", 1);

        let package = register
            .get_package_mut(&self.package_name)
            .unwrap_or_exit_msg("Unable to update symlinked status after creating symlinks", 1);

        package.symlinked = true;

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);

        let styled_message = format!("Successfully linked {}", self.package_name.style()).bold().green();
        println!("{styled_message}");
    }
}

impl LinkArgs {
    /// Checks if linking is allowed and shows a message when it is not allowed or cannot be checked.
    /// Returns true if linking is allowed, false otherwise.
    fn linking_allowed(&self, register: &PackageRegister, package: &InstalledPackage, package_version: &InstalledPackageVersion) -> bool {
        let conflicts = register.get_conflicting_packages(&self.package_name, &package.conflicts_with);
        if !conflicts.is_empty() {
            warning!("The package has conflicts with other packages, cancelling linking");
            println!("Conflicting packages:");
            standard_print::print_list(conflicts.iter().map_styled());
            return false;
        }

        let repository = Repository::new(
            &package_version.metadata_repository_url,
            &package_version.metadata_repository_provider,
        );

        let Some(provider) = provider::create_metadata_provider(&repository) else {
            error!(msg: "Cannot create provider for repository");
            return false;
        };

        let package_version_meta = match provider.read_package_version(&self.package_name, &package.active_version) {
            Ok(package_version_meta) => package_version_meta,
            Err(e) => {
                error!(e, "Unable to read package metadata for package");
                return false;
            },
        };

        // Skip if the package version metadata defines skip_symlinking
        if package_version_meta.skip_symlinking {
            warning!("The package metadata defines we should not symlink this package");
            return false;
        }

        let target_bounds = match package_version_meta.get_best_target(&Target::current()) {
            Ok(target_bounds) => target_bounds,
            Err(e) => {
                error!(e, "The metadata does not contain the current target");
                return false;
            },
        };

        let target = match package_version_meta.get_target(&target_bounds) {
            Ok(target) => target,
            Err(e) => {
                error!(e, "Cannot get current target from package metadata");
                return false;
            },
        };

        // Skip if the package version target metadata defines skip_symlinking
        if let Some(true) = target.skip_symlinking {
            warning!("The package metadata defines we should not symlink this package");
            return false;
        }

        true
    }
}
