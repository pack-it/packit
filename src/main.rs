use crate::commands::execute;
use crate::config::Config;
use crate::repositories::provider::create_repository_provider;

mod commands;
mod config;
mod installer;
mod logger;
mod repositories;
mod target_architecture;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    let repo = config
        .repositories
        .get("core")
        .expect("core repository not in config");

    let provider = create_repository_provider(repo).expect("Cannot create provider");

    execute(&provider).expect("Temporary expect");
}
