// SPDX-License-Identifier: GPL-3.0-only
mod checksum;
mod meta_check;
mod package;
mod portable_repo;

use clap::Subcommand;

use crate::cli::commands::{
    HandleCommand,
    util::{checksum::ChecksumArgs, meta_check::MetaCheckArgs, package::PackageArgs, portable_repo::PortableRepoArgs},
};

/// Provides several utils for advanced users.
#[derive(Subcommand, Debug)]
pub enum UtilArgs {
    /// Calculates the checksum for the file at the given url
    Checksum(ChecksumArgs),

    /// Generates a portable repository containing the specified packages
    PortableRepo(PortableRepoArgs),

    /// Package a package version
    Package(PackageArgs),

    /// Checks the metadata of a package(s) in a repository
    MetaCheck(MetaCheckArgs),
}

impl HandleCommand for UtilArgs {
    /// Handles the util command.
    fn handle(&self) {
        match self {
            Self::Checksum(args) => args.handle(),
            Self::PortableRepo(args) => args.handle(),
            Self::Package(args) => args.handle(),
            Self::MetaCheck(args) => args.handle(),
        }
    }
}
