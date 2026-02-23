use std::{fs, path::Path, str::FromStr};

use crate::{
    cli::display::logging::warning,
    config::Config,
    installer::{
        types::{PackageId, Version},
        Installer, InstallerOptions,
    },
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    utils::symlink::remove_symlinks,
    verifier::{
        error::{Result, VerifierError},
        Issue,
    },
};

/// Repairer which gets issues with the verifier and fixes them.
pub struct Repairer<'a> {
    config: &'a Config,
    manager: &'a RepositoryManager<'a>,
}

impl<'a> Repairer<'a> {
    /// Creates a new repairer.
    pub fn new(config: &'a Config, manager: &'a RepositoryManager) -> Self {
        Self { config, manager }
    }

    /// Fixes the given issue by executing the fix for that issue.
    pub fn fix(&mut self, issue: Issue, register: &mut PackageRegister) -> Result<()> {
        match issue {
            Issue::BrokenTree(missing) => self.fix_broken_tree(missing, register)?,
            Issue::InconsistentStorage(missing) => self.fix_inconsistent_storage(missing, register)?,
            Issue::InconsistentRegister(missing) => self.fix_inconsistent_register(missing, register)?,
            _ => warning!("Fix not executed, because the issue fix is not yet implemented"),
        }

        Ok(())
    }

    /// Fixes broken dependency trees by installing the missing packages.
    fn fix_broken_tree(&mut self, missing: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) -> Result<()> {
        for (_, missing_package) in missing {
            // There could be duplicates in the missing packages, so skip when already seen
            if register.get_package_version(&missing_package).is_some() {
                continue;
            }

            // Install and save
            let installer_options = InstallerOptions::default().skip_symlinking(true);
            let mut installer = Installer::new(&self.config, register, &self.manager, installer_options);
            installer.install(&missing_package.clone().into())?;
        }

        Ok(())
    }

    /// Fixes inconsistent storage by temporarily removing the missing package from the register and then re-installing the packages.
    fn fix_inconsistent_storage(&mut self, missing: Vec<PackageId>, register: &mut PackageRegister) -> Result<()> {
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
            let mut installer = Installer::new(&self.config, register, &self.manager, installer_options);

            installer.install(&missing_package.into())?;
        }

        Ok(())
    }

    /// Fixes an inconsistent register by temporarily removing the missing packages from storage and then re-installing the packages.
    /// Note that it's not possible to recreate the register entries, because some entries like the source repository cannot be defered from the package storage.
    fn fix_inconsistent_register(&mut self, missing: Vec<PackageId>, register: &mut PackageRegister) -> Result<()> {
        let active_directory = self.config.prefix_directory.join("active");
        let bin_directory = self.config.prefix_directory.join("bin");
        let package_directory = self.config.prefix_directory.join("packages");
        for package_id in missing {
            // Figure out the active version
            let active_target = fs::read_link(active_directory.join(&package_id.name))?;
            let target_name = active_target.file_name().ok_or(VerifierError::InvalidSymlink)?;
            let version = Version::from_str(target_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;

            // Check if the package should be the active package when installed
            let active = package_id.version == version;

            // Figure out if symlinked
            let symlinked = !fs::symlink_metadata(bin_directory.join(&package_id.name)).is_err();

            // Temporarily remove the package from the storage
            // We have to uninstall and install, because we can't know the repository source otherwise
            // Remove the symlinks first
            // TODO: Use uninstall scripts here in the future
            if symlinked {
                remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&package_directory))?;
            }

            // Remove the packages
            fs::remove_dir_all(&package_directory.join(&package_id.name).join(package_id.version.to_string()))?;

            // Re-install the package
            let installer_options = InstallerOptions::default().skip_symlinking(!symlinked).skip_active(!active);
            let mut installer = Installer::new(&self.config, register, &self.manager, installer_options);

            installer.install(&package_id.into())?;
        }

        Ok(())
    }
}
