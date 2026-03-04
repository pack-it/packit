use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, repositories::manager::RepositoryManager, storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit,
};

// TODO: Expand command to list updateable packages
#[derive(Args, Debug)]
pub struct ListArgs {}

impl HandleCommand for ListArgs {
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path(config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        for package in register.iterate_all() {
            println!("{}", package.package_id);
        }
    }
}
