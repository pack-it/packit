use crate::{cli::commands, config::Config, repositories::manager::RepositoryManager, utils::constants::CONFIG_DIR};

mod cli;
mod config;
mod installed_packages;
mod installer;
mod repositories;
mod target_architecture;
mod utils;
mod verifier;

fn main() {
    let config = Config::from(CONFIG_DIR).expect("Cannot load config");
    let manager = RepositoryManager::new(&config);
    let cli = commands::Cli::get_instance();

    match cli.handle_command(&manager, &config) {
        Ok(_) => {},
        Err(e) => println!("An error occured: {}\n{:?}", e, e),
    };
}
