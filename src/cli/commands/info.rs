// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    cli::display::{standard_print::DisplayJoined, styled::MapStyled},
    installer::types::PackageName,
    platforms::{DEFAULT_CONFIG_DIR, OsVersion, Target},
    repositories::manager::RepositoryManager,
    utils::packit_version::{current_packit_version, packit_version_name},
};
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
    package: Option<OptionalPackageId>,

    /// True if verbose information should be shown
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// True if displaying package trees as well
    #[arg(long, default_value = "false", requires = "package")]
    tree: bool,

    /// True if the active version should be used
    #[arg(long, default_value = "false", requires = "package")]
    active: bool,
}

impl HandleCommand for InfoArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let Some(package) = &self.package else {
            self.display_packit_info(&config, &register);
            return;
        };

        // Get package information
        let installed_package = match register.get_package(&package.name) {
            Some(package) => package,
            None => not_found::register_package(&package.name, &register),
        };

        // Check if there is version ambiguity (version and `--active` specified)
        if package.version.is_some() && self.active {
            error!(msg: "Version is ambiguous, version and '--active' are both specified");
            exit(1);
        }

        // Get the package version
        let package_version = match &package.version {
            Some(version) => Some(version),
            None if self.active => Some(&installed_package.active_version),
            None => None,
        };

        // Display tree if tree flag is given
        if self.tree {
            let Some(package_version) = package_version else {
                error!(msg: "Displaying a tree requires package version to be specified");
                exit(1);
            };

            let package_id = PackageId::new(package.name.clone(), package_version.clone());
            let tree = match EmptyTree::new_empty(package_id.clone(), &register) {
                Ok(tree) => tree,
                Err(TreeError::NotFound(..)) => not_found::register_package_version(&package_id, &register),
                Err(e) => Err(e).unwrap_or_exit(1),
            };

            println!("{tree}");
            return;
        }

        // Show package version specific information
        if let Some(package_version) = package_version {
            let package_id = PackageId::new(package.name.clone(), package_version.clone());
            self.display_package_version_info(&package_id, &register, installed_package);
            return;
        }

        self.display_package_info(&package.name, installed_package);
    }
}

impl InfoArgs {
    /// Displays information about the Packit installation.
    fn display_packit_info(&self, config: &Config, register: &PackageRegister) {
        let target = Target::current();
        let package_id = PackageId::new(PackageName::packit(), current_packit_version());

        // Read latest version of packit
        let manager = RepositoryManager::new(config);
        let latest_version;
        match manager.read_package(&package_id.name) {
            Ok((repository_id, package_meta)) => match manager.read_latest_supported_version(&repository_id, &package_meta, &target) {
                Ok(version_meta) => latest_version = Some(version_meta.version),
                Err(e) => {
                    error!(e, "Cannot request packit version metadata");
                    latest_version = None;
                },
            },
            Err(e) => {
                error!(e, "Cannot request packit metadata");
                latest_version = None;
            },
        }

        // Create display for new version
        let new_version_disp = match latest_version {
            Some(latest_version) if latest_version > current_packit_version() => format!("yes, {}", latest_version.style()),
            Some(_) | None => "no".into(),
        };

        println!("{}", package_id.style());
        println!("{}", packit_version_name!().italic().cyan());

        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Config directory", DEFAULT_CONFIG_DIR);
        pair_aligner.add("Prefix directory", config.prefix_directory.display());
        pair_aligner.add("Multiuser mode", if config.multiuser { "on" } else { "off" });
        pair_aligner.add("Installed packages", register.iterate_all().count());
        pair_aligner.add("New version available", new_version_disp);
        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();

        println!("Current system");
        let mut pair_aligner = PairAligner::new();

        match &target.os {
            OsVersion::MacOs { version } => {
                pair_aligner.add("OS", format!("macOS {version}"));
            },
            OsVersion::Linux {
                distro,
                distro_version,
                kernel_version,
            } => {
                pair_aligner.add("OS", format!("Linux {distro} {distro_version}"));
                pair_aligner.add("Kernel version", kernel_version);
            },
            OsVersion::Windows { version } => {
                pair_aligner.add("OS", format!("Windows {version}"));
            },
            OsVersion::Unknown => {
                pair_aligner.add("OS", "Unknown".dimmed());
            },
        }

        pair_aligner.add("Architecture", target.architecture);
        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();
    }

    /// Displays package info.
    fn display_package_info(&self, package_name: &PackageName, package: &InstalledPackage) {
        // Sort installed versions for display
        let mut installed_versions: Vec<_> = package.versions.keys().collect();
        installed_versions.sort();

        println!("{}", package_name.style());
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
