use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::Config,
    installer::{
        Symlinker,
        types::{PackageName, Version},
    },
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Switches the active version of the specified package to the specified version.
#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// The name of the package to switch
    pub package_name: PackageName,

    /// The new active version of the package
    pub package_version: Version,

    /// True to skip symlinking the package, false to keep the current symlinked state
    #[arg(long, default_value = "false")]
    pub skip_symlinking: bool,
}

impl HandleCommand for SwitchArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_path = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_path).unwrap_or_exit(1);

        // Get installed package
        let package = register
            .get_package(&self.package_name)
            .unwrap_or_exit_msg(&format!("Package {} is not installed!", self.package_name), 1);

        // Check if the package version is already active
        if package.active_version == self.package_version {
            println!("This version is already the active version");
            return;
        }

        // Get installed package version
        let package_version = package.get_package_version(&self.package_version).unwrap_or_exit_msg(
            &format!("Package {}@{} is not installed!", self.package_name, self.package_version),
            1,
        );

        // Show warning if skip symlinking is specified, but package was symlinked before
        if self.skip_symlinking && package.symlinked {
            warning!(
                "Skipping symlinking while package was symlinked before. The package will not be automatically findable by your system anymore."
            );
        }

        let package_id = package_version.package_id.clone();
        let should_symlink = !self.skip_symlinking && package.symlinked;

        // Set package version to active
        Symlinker::new(&config)
            .set_active(&mut register, &package_id, should_symlink)
            .unwrap_or_exit_msg("Cannot switch active package", 1);

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);

        println!(
            "Successfully switched '{}' active version to {}",
            self.package_name, self.package_version
        )
    }
}
