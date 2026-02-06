use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::{Config, Repository},
    error_handling::HandleError,
    installer::installer::Installer,
    platforms::TARGET_ARCHITECTURE,
    repositories::{manager::RepositoryManager, provider},
    storage::package_register::PackageRegister,
};

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// The name of the package to link
    pub package_name: String,

    /// True to force linking, even when we should not link
    #[arg(short, long, default_value = "false")]
    pub force: bool,
}

impl HandleCommand for LinkArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_path = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_path).unwrap_or_exit(1);

        // Get installed package
        let package = register
            .get_package(&self.package_name)
            .unwrap_or_exit_msg(&format!("Package {} is not installed!", self.package_name), 1);

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
            let repository = Repository {
                path: package_version.source_repository_url.clone(),
                provider: package_version.source_repository_provider.clone(),
            };

            let provider = provider::create_repository_provider(&repository).unwrap_or_exit_msg(
                "Cannot create provider for repository, try --force if you're sure you want to link.",
                1,
            );

            let package_version_meta = provider.read_package_version(&self.package_name, &package.active_version).unwrap_or_exit_msg(
                "Unable to read package metadata for package, try --force if you're sure you want to link.",
                1,
            );

            // Skip if the package version metadata defines skip_symlinking
            if package_version_meta.skip_symlinking {
                println!("The package metadata defines we should not symlink this package, cancelling linking.");
                return;
            }

            let target = package_version_meta.get_target(TARGET_ARCHITECTURE).unwrap_or_exit_msg(
                "The metadata does not contain the current target, try --force if you're sure you want to link.",
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
        Installer::new(config, &mut register, manager)
            .create_symlinks(&install_path)
            .unwrap_or_exit_msg("Unable to link package", 1);

        let package = register
            .get_package_mut(&self.package_name)
            .unwrap_or_exit_msg("Unable to update symlinked status after creating symlinks", 1);

        package.symlinked = true;

        // Save package register
        register.save_to(&register_path).unwrap_or_exit(1);
    }
}
