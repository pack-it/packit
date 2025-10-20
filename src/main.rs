use crate::{
    config::Config,
    repositories::provider::create_repository_provider,
};

mod commands;
mod config;
mod installer;
mod logger;
mod repositories;
mod target_architecture;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    let core_repo = config.repositories.get("core").expect("Core repository not in config");

    let provider = create_repository_provider(core_repo).expect("Cannot create provider");

    commands::handle_command(&provider).expect("Temporary expect");
}
