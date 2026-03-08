use crate::cli::commands::Cli;

mod cli;
mod config;
mod installer;
mod packager;
mod platforms;
mod repositories;
mod storage;
mod utils;
mod verifier;

fn main() {
    Cli::get_instance().handle_command();
}
