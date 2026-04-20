// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    platforms::Target,
    repositories::manager::RepositoryManager,
    storage::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Lists all the installed packages.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// List upgradeable packages
    #[arg(long, default_value = "false")]
    pub upgradeables: bool,
}

impl HandleCommand for ListArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        if !self.upgradeables {
            let mut packages: Vec<&InstalledPackageVersion> = register.iterate_all().collect();
            packages.sort_by(|a, b| a.package_id.to_string().cmp(&b.package_id.to_string()));
            for package in packages {
                println!("{}", package.package_id);
            }

            return;
        }

        let manager = RepositoryManager::new(&config);

        let mut packages: Vec<&InstalledPackageVersion> = register.iterate_all().collect();
        packages.sort_by(|a, b| a.package_id.to_string().cmp(&b.package_id.to_string()));
        let mut upgradeable_found = false;
        for package in packages {
            let (_, package_meta) = manager.read_package(&package.package_id.name).unwrap_or_exit(1);
            let latest_version = package_meta.get_latest_version(&Target::current()).unwrap_or_exit(1);

            if *latest_version == package.package_id.version {
                continue;
            }

            upgradeable_found = true;
            println!("{}", package.package_id);
        }

        if !upgradeable_found {
            println!("No upgradeable packages found");
        }
    }
}
