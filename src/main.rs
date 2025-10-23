use crate::{cli::commands, config::Config, repositories::manager::RepositoryManager};

mod cli;
mod config;
mod installed_packages;
mod installer;
mod repositories;
mod target_architecture;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    let manager = RepositoryManager::new(&config);

    match commands::handle_command(&manager) {
        Ok(_) => {},
        Err(e) => println!("An error occured: {}\n{:?}", e, e),
    };
}
