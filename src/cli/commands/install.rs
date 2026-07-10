// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use colored::Colorize;
use std::process::exit;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{deprecation, logging::error, not_found, standard_print, styled::Styled},
        parameter_checks,
    },
    config::Config,
    installer::{InstallType, Installer, InstallerOptions, types::OptionalPackageId},
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Installs the specified packages, if a version is given that version will be installed,
/// if not the latest available version will be installed. Multiple packages can be specified
/// by entering multiple names, split by a space.
#[derive(Args, Debug)]
pub struct InstallArgs {
    /// The name of the packages to install, with an optional version specified with NAME@VERSION
    #[arg(required = true)]
    pub packages: Vec<OptionalPackageId>,

    /// True to build from source locally, false to use a prebuild version
    #[arg(long, default_value = "false", conflicts_with = "build_all")]
    pub build: bool,

    /// True to build everything from source locally, false to use a prebuild version
    #[arg(long, default_value = "false", conflicts_with = "build")]
    pub build_all: bool,

    /// True to skip symlinking the package, false to use defaults specified for the package
    #[arg(long, default_value = "false")]
    pub skip_symlinking: bool,

    /// True to skip setting the package to active, false to use default behaviour
    #[arg(long, default_value = "false")]
    pub skip_active: bool,

    /// Flag to keep build dependencies after building from source
    #[arg(long, default_value = "false")]
    pub keep_build: bool,

    /// True if verbose information should be shown
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

impl HandleCommand for InstallArgs {
    fn handle(&self) {
        // Check for duplicates, because installing twice will result in a confusing error
        let duplicates = parameter_checks::get_duplicates(&self.packages);
        if !duplicates.is_empty() {
            error!(msg: "Duplicate package arguments are not allowed. The following duplicates were found:");
            standard_print::print_list(duplicates.iter());
            exit(1);
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        // Check if all packages exist before starting installation
        for optional_id in &self.packages {
            let (_, package, package_version) = match manager.read_package_and_version(&optional_id, &Target::current()) {
                Ok(package) => package,
                Err(RepositoryError::PackageNotFoundError { reason, .. }) => {
                    not_found::repository_optional_package(optional_id, &manager, reason)
                },
                Err(e) => {
                    error!(e, "Cannot read package");
                    return;
                },
            };

            // Check if the package or version is deprecated
            deprecation::show_package_warnings(&package);
            deprecation::show_package_version_warnings(&package_version, &optional_id.name);
        }

        // Determine the install type. Note that clap already check if build and build-all are both set (which should not be possible).
        let install_type = if self.build {
            InstallType::Build
        } else if self.build_all {
            InstallType::BuildAll
        } else {
            InstallType::Prebuild
        };

        let installer_options = InstallerOptions::default()
            .install_type(install_type)
            .skip_symlinking(self.skip_symlinking)
            .skip_active(self.skip_active)
            .keep_build(self.keep_build)
            .verbose(self.verbose);
        let mut installer = Installer::new(&config, &mut register, &manager, installer_options);

        // TODO: Check if this exists as an external package (possibly leading to conflicts) (if so, add to external packages)

        // Install all packages
        for optional_id in &self.packages {
            match installer.install(optional_id) {
                Ok(installed_package) => {
                    let styled_message = format!("Successfully installed {}", installed_package.style()).bold().green();
                    println!("{styled_message}");
                },
                Err(error) => error!(error, "Cannot install package {}", optional_id.style()),
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
