use std::process::exit;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::types::{OptionalPackageId, PackageId},
    storage::{installed_package::InstalledPackage, package_register::PackageRegister},
    utils::{tree::EmptyNode, unwrap_or_exit::UnwrapOrExit},
};

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
        let register_dir = PackageRegister::get_default_path(&config);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Get package information
        let package = match register.get_package(&self.package.name) {
            Some(package) => package,
            None => {
                error!(msg: "Package '{}' is not installed", self.package);
                exit(1)
            },
        };

        // Display tree if tree flag is given
        if self.tree {
            if let Some(package_id) = self.package.versioned() {
                let tree = EmptyNode::build_simple_tree(package_id, &register).unwrap_or_exit(1);
                println!("{tree}");
                return;
            } else {
                error!(msg: "Displaying a tree requires package version to be specified.");
                exit(1)
            }
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
        println!("{}", self.package.name);
        println!("{}", package.description);
        if let Some(homepage) = &package.homepage {
            println!("Homepage: {homepage}");
        }

        print!("Installed versions:");
        for version in package.versions.keys() {
            print!(" {version}");
        }
        println!();

        println!("Active version: {}", package.active_version);
        println!("Symlinked: {}", package.symlinked);
    }

    /// Displays the package version info, also checking for the verbose flag for some info.
    fn display_package_version_info(&self, package_id: &PackageId, register: &PackageRegister, package: &InstalledPackage) {
        let package_version = match register.get_package_version(&package_id) {
            Some(package) => package,
            None => {
                error!(msg: "Package '{}' doesn't exist", package_id);
                exit(1)
            },
        };

        println!("{}", package_id);
        println!("{}", package.description);
        if let Some(homepage) = &package.homepage {
            println!("Homepage: {homepage}");
        }

        if let Some(license) = &package_version.license {
            println!("License: {}", license);
        }

        println!(
            "Install path: {}",
            package_version.install_path.to_str().unwrap_or_exit_msg("Invalid install path", 1)
        );

        println!("Active: {}", package.active_version == package_id.version);
        println!("Symlinked: {}", package.symlinked);

        print!("Dependencies: ");
        if package_version.dependencies.is_empty() {
            print!("None");
        }

        for dependency in &package_version.dependencies {
            print!("\n  - {dependency}");
        }
        println!();

        // Early return if verbose is disabled
        if !self.verbose {
            return;
        }

        print!("Dependent: ");
        if package_version.dependents.is_empty() {
            print!("None");
        }

        for dependent in &package_version.dependents {
            print!("\n  - {dependent}");
        }
        println!();

        println!("Source repository provider: {}", package_version.source_repository_provider);
        println!("Source repository url: {}", package_version.source_repository_url);

        print!("Revisions: ");
        if package_version.revisions.is_empty() {
            print!("None");
        }

        for revision in &package_version.revisions {
            print!("\n  - {revision}");
        }

        println!();
    }
}
