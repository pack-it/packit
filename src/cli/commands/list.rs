use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    storage::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Lists all the installed packages.
// TODO: Expand command to list updateable packages
#[derive(Args, Debug)]
pub struct ListArgs;

impl HandleCommand for ListArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let mut packages: Vec<&InstalledPackageVersion> = register.iterate_all().collect();
        packages.sort_by(|a, b| a.package_id.to_string().cmp(&b.package_id.to_string()));
        for package in packages {
            println!("{}", package.package_id);
        }
    }
}
