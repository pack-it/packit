// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::module_inception)]
#![allow(clippy::enum_variant_names)]
mod builder;
mod cli;
mod config;
mod installer;
mod integrity;
mod packager;
mod platforms;
mod register;
mod repositories;
mod utils;

use crate::cli::commands::Cli;

fn main() {
    Cli::get_instance().handle_command();
}
