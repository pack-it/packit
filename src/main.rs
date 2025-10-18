use clap::Parser;

use crate::config::Config;
use crate::commands::Cli;

mod config;
mod commands;

fn main() {
    let command = Cli::parse();
    dbg!(command); // Temporary for simple testing

    let config = Config::from("Config.toml").expect("Cannot load config");
    println!("{config:?}");
}
