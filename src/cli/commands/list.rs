// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display},
    config::Config,
    platforms::Target,
    repositories::manager::RepositoryManager,
    storage::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Lists all the installed packages.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// List updatables packages
    #[arg(long, default_value = "false")]
    pub updatables: bool,
}

impl HandleCommand for ListArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let mut packages: Vec<&InstalledPackageVersion> = register.iterate_all().collect();
        packages.sort_by(|a, b| a.package_id.to_string().cmp(&b.package_id.to_string()));

        if !self.updatables {
            display::print_grid(packages.iter().map(|p| &p.package_id).collect());

            return;
        }

        let manager = RepositoryManager::new(&config);
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
