use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    installer::{Installer, InstallerOptions, types::OptionalPackageId},
    platforms::TARGET_ARCHITECTURE,
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

// TODO: Add a version parameter to specify which version to update or downgrade to
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// The name of the packages to update, with an optional version specified with NAME@VERSION
    optional_id: OptionalPackageId,
}

impl HandleCommand for UpdateArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path(config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let (_, package_meta) = manager.read_package(&self.optional_id.name).unwrap_or_exit(1);
        let latest_version = package_meta.get_latest_version(TARGET_ARCHITECTURE).unwrap_or_exit(1);

        let options = InstallerOptions::default().skip_active(true).skip_symlinking(true);
        let mut installer = Installer::new(config, &mut register, manager, options);

        installer.update(&self.optional_id, latest_version).unwrap_or_exit(1);

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
