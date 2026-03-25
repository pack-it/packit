use std::process::exit;

use clap::Args;

use crate::{
    cli::{commands::HandleCommand, display::logging::error},
    config::Config,
    installer::{InstallType, Installer, InstallerOptions, types::OptionalPackageId},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::{duplicates, unwrap_or_exit::UnwrapOrExit},
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
}

impl HandleCommand for InstallArgs {
    fn handle(&self) {
        // Check for duplicates, because installing twice will result in a confusing error
        let duplicates = duplicates::get_duplicates(&self.packages);
        if !duplicates.is_empty() {
            let mut duplicate_string = String::new();
            for duplicate in duplicates {
                duplicate_string.push_str(&duplicate.to_string());
            }

            error!(msg: "Duplicate package arguments are not allowed. The following duplicates were found: {duplicate_string}");
            exit(1);
        }

        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        let register_dir = PackageRegister::get_default_path(&config);
        let mut register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

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
            .keep_build(self.keep_build);
        let mut installer = Installer::new(&config, &mut register, &manager, installer_options);

        // TODO: Check if this exists as an external package (possibly leading to conflicts) (if so, add to external packages)

        // Install all packages
        for package_id in &self.packages {
            if let Err(error) = installer.install(&package_id) {
                error!(error, "Cannot install package {package_id}");
            }
        }

        // Save changes
        register.save_to(&register_dir).unwrap_or_exit(1);
    }
}
