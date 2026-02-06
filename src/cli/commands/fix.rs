use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::Config,
    repositories::manager::RepositoryManager,
};

#[derive(Args, Debug)]
pub struct FixArgs;

impl HandleCommand for FixArgs {
    fn handle(&self, _: &Config, _: &RepositoryManager) {
        warning!("This command is not yet fully implemented");
    }
}
