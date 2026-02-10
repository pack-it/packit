use clap::Args;

use crate::{
    cli::commands::HandleCommand, config::Config, repositories::manager::RepositoryManager, storage::package_register::PackageRegister,
    utils::unwrap_or_exit::UnwrapOrExit, verifier::Verifier,
};

#[derive(Args, Debug)]
pub struct CheckArgs;

impl HandleCommand for CheckArgs {
    fn handle(&self, config: &Config, _: &RepositoryManager) {
        let register_dir = PackageRegister::get_default_path();
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);
        let verifier = Verifier::new(config, &register);
        let issues = verifier.find_issues().unwrap_or_exit(1);

        if issues.is_empty() {
            println!("No issues were found");
            return;
        }

        for issue in issues {
            print!("{}\n", issue);
        }

        println!("Consider running `pit fix` to resolve the issues above.")
    }
}
