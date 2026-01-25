use crate::{cli::commands::Cli, config::Config, repositories::manager::RepositoryManager};

mod cli;
mod config;
mod error_handling;
mod installed_packages;
mod installer;
mod platforms;
mod repositories;
mod utils;
mod verifier;

fn main() {
    let config = Config::from(&Config::get_default_path()).expect("Cannot load config");
    let manager = RepositoryManager::new(&config);
    Cli::get_instance().handle_command(&manager, &config);
}
