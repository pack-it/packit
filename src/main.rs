use crate::{cli::commands, config::Config, installer::installer::Installer, repositories::manager::RepositoryManager};

mod cli;
mod config;
mod installed_packages;
mod installer;
mod repositories;
mod target_architecture;
mod verifier;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    let manager = RepositoryManager::new(&config);
    let installer = Installer::new(&config);

    match commands::handle_command(&manager, &installer) {
        Ok(_) => {},
        Err(e) => println!("An error occured: {}\n{:?}", e, e),
    };
}
