// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::Spinner},
    config::Config,
    installer::types::PackageId,
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::{manager::RepositoryManager, portable_repo::PortableRepoCreator},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Generates a portable repository containing the specified packages
#[derive(Args, Debug)]
pub struct PortableRepoArgs {
    /// The destination directory
    destination: PathBuf,

    /// The packages to include in the portable repository
    packages: Vec<PackageId>,

    /// True to exclude prebuilds in the portable repository
    #[arg(long, default_value = "false")]
    exclude_prebuilds: bool,

    /// True to skip automatic addition of dependencies to the portable repository
    #[arg(long, default_value = "false")]
    skip_dependency_resolution: bool,
}

impl HandleCommand for PortableRepoArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register =
            PackageRegister::from(&PackageRegister::get_path(&config.prefix_directory)).unwrap_or_exit_msg("Cannot load register", 1);

        let spinner = Spinner::new("Generating portable repository".into());
        spinner.show();

        let creator = PortableRepoCreator::new(
            &config,
            &manager,
            &register,
            Target::current(),
            self.exclude_prebuilds,
            self.skip_dependency_resolution,
        );

        creator
            .create_portable_repo(self.packages.iter().cloned().collect(), &self.destination)
            .unwrap_or_exit_msg("Cannot create portable repository", 1);

        spinner.finish();
        println!("Created portable repository at {}!", self.destination.display());
    }
}
