// SPDX-License-Identifier: GPL-3.0-only
use clap::Args;
use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{
    cli::{
        commands::HandleCommand,
        display::{Spinner, logging::error, not_found, styled::Styled},
        parameter_checks,
    },
    config::{Config, Repository},
    installer::types::{OptionalPackageId, PackageId},
    packager,
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::{provider, types::PrebuildsList},
    utils::unwrap_or_exit::UnwrapOrExit,
};

/// Packages the specified package into a prebuild and store it in the destination directory, together with a checksum of the prebuild.
#[derive(Args, Debug)]
pub struct PackageArgs {
    /// Destination of the compressed package
    destination: PathBuf,

    /// The ids of the packages to package
    packages: Vec<PackageId>,

    /// True to structure the package into a prebuild directory
    #[arg(short, long, default_value = "false")]
    structured: bool,

    /// True to package all installed packages
    #[arg(short, long, default_value = "false", conflicts_with = "packages")]
    all: bool,

    /// Exclude packages when using the `--all` flag, specified with <PACKAGE-NAME> ...
    // Note that `num_args = 1..` is needed to consume all 'non-flag' values after the exclude
    #[arg(long, requires = "all", num_args = 1..)]
    exclude: Vec<OptionalPackageId>,
}

impl HandleCommand for PackageArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let register_dir = PackageRegister::get_path(&config.prefix_directory);
        let register = PackageRegister::from(&register_dir).unwrap_or_exit(1);

        let packages: Vec<&PackageId> = match self.all {
            true => register.iterate_all().map(|p| &p.package_id).collect(),
            false => self.packages.iter().collect(),
        };

        // Exclude packages (note that the default value of exclude is an empty vec, so nothing is filtered)
        let packages: Vec<&PackageId> = packages.into_iter().filter(|p| !parameter_checks::contains_package_id(&self.exclude, p)).collect();

        if packages.is_empty() {
            error!(msg: "Nothing packaged, no packages specified");
            exit(1);
        }

        for package_id in packages {
            // Get the correct install directory
            let destination = match self.structured {
                true => {
                    let prefix = package_id.name.get_prefix().to_string();
                    &self.destination.join("packages").join(prefix).join(&package_id.name).join(package_id.version.to_string())
                },
                false => &self.destination,
            };

            self.package(package_id, destination, &config, &register);
        }
    }
}

impl PackageArgs {
    /// Packages a specific package to the given destination.
    fn package(&self, package_id: &PackageId, destination: &Path, config: &Config, register: &PackageRegister) {
        let package_version = match register.get_package_version(package_id) {
            Some(package_version) => package_version,
            None => not_found::register_package_version(package_id, register),
        };

        // TODO: should we also store prebuilds.toml, or require the repository to be available?
        // Create metadata provider
        let repository = Repository::new(
            &package_version.metadata_repository_url,
            &package_version.metadata_repository_provider,
        );
        let Some(provider) = provider::create_metadata_provider(&repository) else {
            error!(msg: "Cannot create provider for {}, skipping packaging", package_id.style());
            return;
        };

        // Request package metadata
        let package_meta = match provider.read_package(&package_id.name) {
            Ok(package_meta) => package_meta,
            Err(e) => {
                error!(e, "Cannot read package metadata of {}, skipping packaging", package_id.style());
                return;
            },
        };

        // Request prebuilds list
        let prebuilds_list = match provider.read_prebuilds_list(&package_id.name, &package_id.version) {
            Ok(Some(prebuilds_list)) => prebuilds_list,
            Ok(None) => PrebuildsList::default(package_meta.supported_versions.keys()),
            Err(e) => {
                error!(e, "Cannot read prebuild list for {}, skipping packaging", package_id.style());
                return;
            },
        };

        // Retrieve `prebuild_id` to use
        let Some((prebuild_id, prebuild_meta)) = prebuilds_list.get_best_prebuild(&Target::current()) else {
            error!(msg: "Cannot find prebuild to create for {}, skipping packaging", package_id.style());
            return;
        };

        // Automatically create the destination directory
        fs::create_dir_all(destination).unwrap_or_exit_msg("Failed to create prebuild directory", 1);

        // Call packager and show spinner
        let spinner_message = format!("Packaging {} to '{}'", package_id.style(), destination.display());
        let spinner = Spinner::new(spinner_message);
        spinner.show();
        let revisions = package_version.revisions.len() as u64;
        packager::package(config, package_id, destination, revisions, prebuild_id, prebuild_meta).unwrap_or_exit(1);
        spinner.finish();
    }
}
