// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display},
    config::Config,
    platforms::Target,
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
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let mut packages: Vec<&InstalledPackageVersion> = register.iterate_all().collect();
        packages.sort_by_key(|a| a.package_id.to_string());

        if self.updatables {
            self.updatables_list(packages, &config);

            return;
        }

        if self.active {
            display::print_grid(register.iterate_active_packages().collect());

            return;
        }

        display::print_grid(packages.iter().map(|p| &p.package_id).collect());
    }
}

impl ListArgs {
    /// Lists all updatable packages.
    fn updatables_list(&self, packages: Vec<&InstalledPackageVersion>, config: &Config) {
        let manager = RepositoryManager::new(config);
        let mut updatables = Vec::new();
        for package in packages {
            let (_, package_meta) = manager.read_package(&package.package_id.name).unwrap_or_exit(1);
            let latest_version = package_meta.get_latest_version(&Target::current()).unwrap_or_exit(1);

            if *latest_version == package.package_id.version {
                continue;
            }

            updatables.push(&package.package_id);
        }

        if updatables.is_empty() {
            println!("No updatable packages found");
            return;
        }

        display::print_grid(updatables);
    }
}
