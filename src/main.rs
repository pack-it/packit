use crate::config::Config;

mod config;

fn main() {
    let config = Config::from("Config.toml").expect("Cannot load config");

    println!("{config:?}");
}
