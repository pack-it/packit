// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::Path, str::FromStr};

use crate::{
    cli::display::logging::warning,
    config::{Config, EditableConfig},
    installer::{
        Installer, InstallerOptions,
        types::{PackageId, Version},
    },
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::io::remove_symlinks,
    verifier::{
        Issue,
        error::{Result, VerifierError},
    },
};

/// Repairer that fixes issues found by the verifier.
pub struct Repairer;

impl Repairer {
    /// Creates a new repairer.
    pub fn new() -> Self {
        Self
    }

    pub fn fix_initial_issues(&mut self, issue: Issue) -> Result<()> {
        match issue {
            Issue::MissingConfig => self.fix_missing_config()?,

            _ => warning!("Fix not executed, because it is not an initial issue"),
        }

        Ok(())
    }

    /// Fixes the given issue by executing the fix for that issue.
    /// Note: The register is not saved after the fix is applied.
    pub fn fix(&mut self, issue: Issue, register: &mut PackageRegister, config: &Config, manager: &RepositoryManager) -> Result<()> {
        match issue {
            Issue::BrokenTree(missing) => self.fix_broken_tree(missing, register, config, manager)?,
            Issue::InconsistentStorage(missing) => self.fix_inconsistent_storage(missing, register, config, manager)?,
            Issue::InconsistentRegister(missing) => self.fix_inconsistent_register(missing, register, config, manager)?,
            _ => warning!("Fix not executed, because the issue fix is not yet implemented"),
        }

        Ok(())
    }

    /// Fixes a missing Config.toml. Either by rebuilding the config from known information or using default values.
    /// TODO: Implement rebuilding config from known data
    fn fix_missing_config(&self) -> Result<()> {
        // Create a default config and adjust when fields can be recovered so new config fields don't create bugs
        EditableConfig::default()?.save_to(&Config::get_default_path())?;
        Ok(())
    }

    /// Fixes broken dependency trees by installing the missing packages.
    fn fix_broken_tree(
        &self,
        missing: Vec<(PackageId, PackageId)>,
        register: &mut PackageRegister,
        config: &Config,
        manager: &RepositoryManager,
    ) -> Result<()> {
        for (_, missing_package) in missing {
            // There could be duplicates in the missing packages, so skip when already seen
            if register.get_package_version(&missing_package).is_some() {
                continue;
            }

            // Install the package
            let installer_options = InstallerOptions::default().skip_symlinking(true);
            let mut installer = Installer::new(config, register, manager, installer_options);
            installer.install(&missing_package.clone().into())?;
        }

        Ok(())
    }

    /// Fixes inconsistent storage by temporarily removing the missing package from the register and then re-installing the packages.
    fn fix_inconsistent_storage(
        &mut self,
        missing: Vec<PackageId>,
        register: &mut PackageRegister,
        config: &Config,
        manager: &RepositoryManager,
    ) -> Result<()> {
        for missing_package in missing {
            // Gather the package settings before removing the package from the register
            let (symlinked, active) = match register.get_package(&missing_package.name) {
                Some(package) => (package.symlinked, package.active_version == missing_package.version),
                None => {
                    warning!("Inconsistent package cannot be found in Installed.toml anymore, eventhough it was found before.");
                    (false, false)
                },
            };

            // Temporarily remove the package from the register
            register.remove_package_version(&missing_package);

            let installer_options = InstallerOptions::default().skip_symlinking(!symlinked).skip_active(!active);
            let mut installer = Installer::new(config, register, manager, installer_options);

            installer.install(&missing_package.into())?;
        }

        Ok(())
    }

    // TODO: Somehow store the package info somewhere before deleting, so package history is preserved
    /// Fixes an inconsistent register by temporarily removing the missing packages from storage and then re-installing the packages.
    /// Note that it's not possible to recreate the register entries, because some entries like the source repository cannot be defered from the package storage.
    fn fix_inconsistent_register(
        &mut self,
        missing: Vec<PackageId>,
        register: &mut PackageRegister,
        config: &Config,
        manager: &RepositoryManager,
    ) -> Result<()> {
        let active_directory = config.prefix_directory.join("active");
        let bin_directory = config.prefix_directory.join("bin");
        let package_directory = config.prefix_directory.join("packages");
        for package_id in missing {
            // Figure out the active version
            let active_target = fs::read_link(active_directory.join(&package_id.name))?;
            let target_name = active_target.file_name().ok_or(VerifierError::InvalidSymlink)?;
            let version = Version::from_str(target_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;

            // Check if the package should be the active package when installed
            let active = package_id.version == version;

            // Figure out if symlinked
            let symlinked = fs::symlink_metadata(bin_directory.join(&package_id.name)).is_ok();

            // Temporarily remove the package from the storage
            // We have to uninstall and install, because we can't know the repository source otherwise
            // Remove the symlinks first
            if symlinked {
                remove_symlinks(Path::new(&config.prefix_directory), &package_directory)?;
            }

            // Remove the package
            fs::remove_dir_all(&package_directory.join(&package_id.name).join(package_id.version.to_string()))?;

            // Re-install the package
            let installer_options = InstallerOptions::default().skip_symlinking(!symlinked).skip_active(!active);
            let mut installer = Installer::new(config, register, manager, installer_options);

            installer.install(&package_id.into())?;
        }

        Ok(())
    }
}
