use crate::{cli::commands, config::Config, repositories::manager::RepositoryManager};

mod cli;
mod config;
mod dependency;
mod installed_packages;
mod installer;
mod platforms;
mod repositories;
mod utils;
mod verifier;
mod version;

fn main() {
    let config = Config::from(&Config::get_default_path()).expect("Cannot load config");
    let manager = RepositoryManager::new(&config);
    let cli = commands::Cli::get_instance();

    match cli.handle_command(&manager, &config) {
        Ok(_) => {},
        Err(e) => println!("An error occured: {}\n{:?}", e, e),
    };
}
