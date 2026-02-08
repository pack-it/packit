use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, repositories::manager::RepositoryManager, storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit, verifier::get_packages,
};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Flag to indicate a full check (actually check packit install directory)
    #[arg(short, long)]
    use_dir: bool,
}

impl HandleCommand for ListArgs {
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        if self.use_dir {
            for package in get_packages(&config).unwrap_or_exit(1) {
                println!("{}", package);
            }
        } else {
            for package in register.iterate_all() {
                println!("{}", package.package_id);
            }
        }
    }
}
