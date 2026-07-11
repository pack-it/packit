// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{
            self,
            styled::{MapStyled, Styled},
        },
    },
    config::Config,
    installer::{
        Installer, InstallerOptions,
        types::{OptionalPackageId, PackageId},
    },
    register::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Lists all the installed packages.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// List updatables packages
    #[arg(long, default_value = "false", conflicts_with = "active")]
    pub updatables: bool,

    /// List active packages
    #[arg(long, default_value = "false", conflicts_with = "updatables")]
    pub active: bool,
}

impl HandleCommand for ListArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        if self.updatables {
            self.updatables_list(&config, &mut register);

            return;
        }

        if self.active {
            display::print_grid(&register.iterate_active_packages().map_styled().collect());

            return;
        }

        let mut packages: Vec<&InstalledPackageVersion> = register.iterate_all().collect();
        packages.sort_by_key(|a| a.package_id.to_string());
        display::print_grid(&packages.iter().map(|p| p.package_id.style()).collect());
    }
}

impl ListArgs {
    /// Lists all updatable packages.
    fn updatables_list(&self, config: &Config, register: &mut PackageRegister) {
        let manager = RepositoryManager::new(config);
        let options = InstallerOptions::default();
        let installer = Installer::new(config, register, &manager, options);

        let updatables = installer.get_updatables().unwrap_or_exit(1);
        if updatables.is_empty() {
            println!("All packages are up-to-date!");
            return;
        }

        // Expand the updatables with all installed versions instead of just the highest one
        let mut expanded_updatables = Vec::new();
        for package in updatables {
            for version in register.get_package(&package.name).expect("Expected package to exist").versions.keys() {
                let package_id = PackageId::new(package.name.clone(), version.clone());
                let optional_id = OptionalPackageId::from(package_id);
                expanded_updatables.push(optional_id.style());
            }
        }

        display::print_grid(&expanded_updatables);
    }
}
