// SPDX-License-Identifier: GPL-3.0-only
use std::process::exit;

use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::logging::{error, warning},
    },
    config::{Config, Repository},
    installer::{Symlinker, types::PackageName},
    platforms::Target,
    repositories::{provider, types::PackageVersionMeta},
    storage::package_register::PackageRegister,
    utils::{fuzzy, unwrap_or_exit::UnwrapOrExit},
};

/// Links the specified package into the /bin, /lib, /share, etc. directories.
#[derive(Args, Debug)]
pub struct LinkArgs {
    /// The name of the package to link
    pub package_name: PackageName,

    /// True to force linking, even when we should not link
    #[arg(short, long, default_value = "false")]
    pub force: bool,
}

impl HandleCommand for LinkArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_path = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_path).unwrap_or_exit(1);

        // Get installed package
        let package = match register.get_package(&self.package_name) {
            Some(package) => package,
            None => {
                error!(msg: "Package {} is not installed!", self.package_name);

                let fuzzy_match = fuzzy::min_search(register.iterate_package_names(), &self.package_name);
                if let Some(fuzzy_match) = fuzzy_match {
                    println!("Did you mean: '{fuzzy_match}'?");
                }

                exit(1);
            },
        };

        // Check if the package is already symlinked
        if package.symlinked {
            println!("This package is already symlinked");
            return;
        }

        // Show warning if forced
        if self.force {
            warning!("Forcing symlink can cause problems, please be carefull when using --force");
        }

        // Get active package version
        let package_version = package
            .get_package_version(&package.active_version)
            .unwrap_or_exit_msg("Unable to retrieve active version of package", 1);

        // Check if we are allowed to symlink when not forcing
        if !self.force {
            let repository = Repository::new(&package_version.source_repository_url, &package_version.source_repository_provider);

            let provider = provider::create_metadata_provider(&repository).unwrap_or_exit_msg(
                "Cannot create provider for repository, try --force if you're sure you want to link.",
                1,
            );

            let package_version_meta: PackageVersionMeta =
                provider.read_package_version(&self.package_name, &package.active_version).unwrap_or_exit_msg(
                    "Unable to read package metadata for package, try --force if you're sure you want to link.",
                    1,
                );

            // Skip if the package version metadata defines skip_symlinking
            if package_version_meta.skip_symlinking {
                println!("The package metadata defines we should not symlink this package, cancelling linking.");
                return;
            }

            let target_bounds = package_version_meta.get_best_target(&Target::current()).unwrap_or_exit_msg(
                "The metadata does not contain the current target, try --force if you're sure you want to link.",
                1,
            );

            let target = package_version_meta.get_target(&target_bounds).unwrap_or_exit_msg(
                "Cannot get current target from package metadata, try --force if you're sure you want to link.",
                1,
            );

            // Skip if the package version target metadata defines skip_symlinking
            if let Some(true) = target.skip_symlinking {
                println!("The package metadata defines we should not symlink this package, cancelling linking.");
                return;
            }
        }

        let install_path = package_version.install_path.clone();

        // Create symlinks
        Symlinker::new(&config).create_symlinks(&install_path).unwrap_or_exit_msg("Unable to link package", 1);

        let package = register
            .get_package_mut(&self.package_name)
            .unwrap_or_exit_msg("Unable to update symlinked status after creating symlinks", 1);

        package.symlinked = true;

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);

        println!("Successfully linked {}", self.package_name);
    }
}
