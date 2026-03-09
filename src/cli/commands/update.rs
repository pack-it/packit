use std::process::exit;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{
        Installer, InstallerOptions,
        types::{OptionalPackageId, Version},
    },
    platforms::Target,
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// The name of the package to update, with an optional version specified with NAME@VERSION.
    optional_id: OptionalPackageId,

    /// The version to update to. This can only be a higher version than the current version.
    new_version: Option<Version>,
}

impl HandleCommand for UpdateArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let (_, package_meta) = manager.read_package(&self.optional_id.name).unwrap_or_exit(1);

        let new_version = match &self.new_version {
            Some(version) => version,
            None => package_meta.get_latest_version(&Target::current()).unwrap_or_exit(1),
        };

        // Check if new version exists
        if !package_meta.versions.contains(new_version) {
            error!(msg: "New package version '{new_version}' does not exist.");
            exit(1);
        }

        let options = InstallerOptions::default().skip_active(true).skip_symlinking(true);
        let mut installer = Installer::new(&config, &mut register, &manager, options);

        installer.update(&self.optional_id, new_version).unwrap_or_exit(1);

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
