use clap::Parser;

use crate::config::Config;
use crate::commands::Cli;
use crate::repositories::provider::{create_repository_provider, RepositoryProvider};

mod config;
mod commands;
mod repositories;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    println!("{config:?}");

    let repo = config.repositories.get("core").expect("core repository not in config");
    let provider = create_repository_provider(repo).expect("Cannot create provider");
    
    println!("Repository metadata: {:?}", provider.read_repository_metadata());
    println!("Package htop: {:?}", provider.read_package("htop".into()));
    println!("Package htop 3.4.1: {:?}", provider.read_package_version("htop".into(), "3.4.1".into()));

    let command = Cli::parse();
    dbg!(command); // Temporary for simple testing
}
