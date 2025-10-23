use crate::{
    cli::commands,
    config::Config,
    repositories::{manager::RepositoryManager, provider::create_repository_provider},
};

mod cli;
mod config;
mod installed_packages;
mod installer;
mod repositories;
mod target_architecture;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    let core_repo = config.repositories.get("core").expect("Core repository not in config");

    let provider = create_repository_provider(core_repo).expect("Cannot create provider");

    let manager = RepositoryManager::new(&config);

    match commands::handle_command(&provider) {
        Ok(_) => {},
        Err(e) => println!("An error occured: {}\n{:?}", e, e),
    };
}
