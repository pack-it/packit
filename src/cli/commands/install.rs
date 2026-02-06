use clap::Args;

use crate::{
    cli::commands::HandleCommand,
    config::Config,
    error_handling::HandleError,
    installer::{installer::Installer, types::Version},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The name of the package to install
    pub package_name: String,

    /// The version of the package to install
    #[arg(short, long)]
    pub version: Option<Version>,

    /// Flag to keep build dependencies after building from source
    #[arg(long, default_value = "false")]
    pub keep_build: bool,
}

impl HandleCommand for InstallArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // TODO: Check if this exists as an external package (possibly leading to conflicts) (if so, add to external packages)

        Installer::new(&config, &mut register, &manager)
            .install(&self.package_name, self.version.as_ref(), self.keep_build)
            .unwrap_or_exit(1);

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
