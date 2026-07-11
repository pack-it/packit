// SPDX-License-Identifier: GPL-3.0-only
#![warn(clippy::doc_markdown, clippy::inconsistent_struct_constructor, clippy::derive_partial_eq_without_eq)]
#![warn(clippy::cast_lossless, clippy::cargo_common_metadata, clippy::perf, clippy::complexity, clippy::suspicious)]
#![allow(clippy::module_inception)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::collapsible_if)] // Allowed because of rustfmt formatting incompatibilities
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
