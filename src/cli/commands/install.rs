use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{types::OptionalPackageId, Installer, InstallerOptions},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The name of the package to install, with an optional version specified with NAME@VERSION
    #[arg(num_args(0..))]
    pub packages: Vec<OptionalPackageId>,

    /// True to build from source locally, false to use a prebuild version
    #[arg(long, default_value = "false")]
    pub build: bool,

    /// True to skip symlinking the package, false to use defaults specified for the package
    #[arg(long, default_value = "false")]
    pub skip_symlinking: bool,

    /// True to skip setting the package to active, false to use default behaviour
    #[arg(long, default_value = "false")]
    pub skip_active: bool,

    /// Flag to keep build dependencies after building from source
    #[arg(long, default_value = "false")]
    pub keep_build: bool,
}

impl HandleCommand for InstallArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let installer_options = InstallerOptions::default()
            .build_source(self.build)
            .skip_symlinking(self.skip_symlinking)
            .skip_active(self.skip_active)
            .keep_build(self.keep_build);
        let mut installer = Installer::new(&config, &mut register, &manager, installer_options);

        // TODO: Check if this exists as an external package (possibly leading to conflicts) (if so, add to external packages)
        // TODO: check for duplicate packages in Vec

        // Install all packages
        for package_id in &self.packages {
            if let Err(error) = installer.install(&package_id) {
                error!(error, "Cannot install package {package_id}");
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
