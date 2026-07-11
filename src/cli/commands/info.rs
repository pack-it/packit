// SPDX-License-Identifier: GPL-3.0-only
use crate::cli::display::{standard_print::DisplayJoined, styled::MapStyled};
use clap::Args;
use colored::Colorize;
use std::process::exit;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{
            aligned_print::PairAligner,
            logging::error,
            not_found,
            standard_print::{self, DisplayOption},
            styled::Styled,
        },
    },
    config::Config,
    installer::types::{OptionalPackageId, PackageId},
    register::{installed_package::InstalledPackage, package_register::PackageRegister},
    utils::{
        tree::{EmptyTree, TreeError},
        unwrap_or_exit::UnwrapOrExit,
    },
};

/// Shows info about the specified installed package.
#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Optional package id
    package: OptionalPackageId,

    /// True if verbose information should be shown
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// True if displaying package trees as well
    #[arg(long, default_value = "false")]
    tree: bool,
}

impl HandleCommand for InfoArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Get package information
        let package = match register.get_package(&self.package.name) {
            Some(package) => package,
            None => not_found::register_package(&self.package.name, &register),
        };

        // Display tree if tree flag is given
        if self.tree {
            let Some(package_id) = self.package.versioned() else {
                error!(msg: "Displaying a tree requires package version to be specified.");
                exit(1);
            };

            let tree = match EmptyTree::new_empty(package_id.clone(), &register) {
                Ok(tree) => tree,
                Err(TreeError::NotFound(..)) => not_found::register_package_version(&package_id, &register),
                Err(e) => Err(e).unwrap_or_exit(1),
            };

            println!("{tree}");
            return;
        }

        // Show package version specific information
        if let Some(package_id) = self.package.versioned() {
            self.display_package_version_info(&package_id, &register, package);
            return;
        }

        self.display_package_info(package);
    }
}

impl InfoArgs {
    /// Displays package info.
    fn display_package_info(&self, package: &InstalledPackage) {
        // Sort installed versions for display
        let mut installed_versions: Vec<_> = package.versions.keys().collect();
        installed_versions.sort();

        println!("{}", self.package.name.style());
        println!("{}", package.description.italic().cyan());

        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Homepage", package.homepage.display());
        pair_aligner.add("Installed versions", installed_versions.iter().map_styled().display(" | "));
        pair_aligner.add("Active version", &package.active_version);
        pair_aligner.add("Symlinked", if package.symlinked { "yes" } else { "no" });
        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();

        if self.verbose {
            print!("Conflicts with: ");
            standard_print::print_list_or_none(package.conflicts_with.iter());
        }
    }

    /// Displays the package version info, also checking for the verbose flag for some info.
    fn display_package_version_info(&self, package_id: &PackageId, register: &PackageRegister, package: &InstalledPackage) {
        let package_version = match register.get_package_version(package_id) {
            Some(package) => package,
            None => not_found::register_package_version(package_id, register),
        };

        println!("{}", package_id.style());
        println!("{}", package.description.italic().cyan());

        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Homepage", package.homepage.display());
        pair_aligner.add("License", package_version.license.style());
        pair_aligner.add("Install path", package_version.install_path.display());
        pair_aligner.add("Active", if package.active_version == package_id.version { "yes" } else { "no" });
        pair_aligner.add("Symlinked", if package.symlinked { "yes" } else { "no" });

        if self.verbose {
            pair_aligner.add("Metadata repository provider", &package_version.metadata_repository_provider);
            pair_aligner.add("Metadata repository url", &package_version.metadata_repository_url);
        }

        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();

        print!("Dependencies: ");
        standard_print::print_list_or_none(package_version.dependencies.iter().map(|d| d.style()));

        // Early return if verbose is disabled
        if !self.verbose {
            return;
        }

        print!("Dependents: ");
        standard_print::print_list_or_none(package_version.dependents.iter().map(|d| d.style()));

        print!("Revisions: ");
        standard_print::print_list_or_none(package_version.revisions.iter());
    }
}
