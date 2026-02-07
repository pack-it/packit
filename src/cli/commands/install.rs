use std::str::FromStr;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    error_handling::HandleError,
    installer::{types::PackageId, Installer},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The name of the package to install, with an optional version specified with NAME@VERSION
    #[arg(num_args(0..))]
    pub packages: Vec<String>,

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

        let mut installer = Installer::new(&config, &mut register, &manager);

        // TODO: Check if this exists as an external package (possibly leading to conflicts) (if so, add to external packages)

        let mut packages = Vec::new();

        // Convert packages into package name and version
        for package in &self.packages {
            if package.contains("@") {
                let package_id =
                    PackageId::from_str(&package).unwrap_or_exit_msg(&format!("Package '{package}' is not a valid package identifier."), 1);

                packages.push((package_id.name, Some(package_id.version)));
                continue;
            }

            packages.push((package.into(), None));
        }

        // TODO: check for duplicate packages in Vec

        // Install all packages
        for (package_name, version) in packages {
            if let Err(error) = installer.install(
                &package_name,
                version.as_ref(),
                self.build,
                self.skip_symlinking,
                self.skip_active,
                self.keep_build,
            ) {
                error!(error, "Cannot install package {package_name}");
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
