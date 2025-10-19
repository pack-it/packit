use clap::Parser;

use crate::config::Config;
use crate::commands::Cli;
use crate::repositories::provider::create_provider;
use crate::repositories::repository::{read_package, read_package_version, read_repository_metadata};

mod config;
mod commands;
mod repositories;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");
    println!("{config:?}");

    let provider = create_provider(config.repositories.get("core").expect("core repository not in config")).expect("Cannot create provider");
    
    println!("Repository metadata: {:?}", read_repository_metadata(&provider));
    println!("Package htop: {:?}", read_package(&provider, "htop".into()));
    println!("Package htop 3.4.1: {:?}", read_package_version(&provider, "htop".into(), "3.4.1".into()));

    let command = Cli::parse();
    dbg!(command); // Temporary for simple testing
}
