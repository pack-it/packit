// SPDX-License-Identifier: GPL-3.0-only
mod checksum;
mod portable_repo;

use clap::Subcommand;

use crate::cli::commands::{
    HandleCommand,
    util::{checksum::ChecksumArgs, portable_repo::PortableRepoArgs},
};

/// Provides several utils for advanced users.
#[derive(Subcommand, Debug)]
pub enum UtilArgs {
    /// Calculates the checksum for the file at the given url
    Checksum(ChecksumArgs),

    /// Generates a portable repository containing the specified packages
    PortableRepo(PortableRepoArgs),
}

impl HandleCommand for UtilArgs {
    /// Handles the util command.
    fn handle(&self) {
        match self {
            Self::Checksum(args) => args.handle(),
            Self::PortableRepo(args) => args.handle(),
        }
    }
}
