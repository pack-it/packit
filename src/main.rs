// SPDX-License-Identifier: GPL-3.0-only
use crate::cli::commands::Cli;

mod cli;
mod config;
mod installer;
mod packager;
mod platforms;
mod register;
mod repositories;
mod utils;
mod verifier;

fn main() {
    Cli::get_instance().handle_command();
}
