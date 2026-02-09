use std::path::PathBuf;

use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, installer::types::PackageId, packager::package, repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};

#[derive(Args, Debug)]
pub struct PackageArgs {
    /// The package id of the package
    pub package_id: PackageId,

    /// Destination of the compressed package
    pub destination: PathBuf,
}

impl HandleCommand for PackageArgs {
    fn handle(&self, config: &Config, manager: &RepositoryManager) {
        package(config, &self.package_id, &self.destination, manager).unwrap_or_exit(1);
    }
}
