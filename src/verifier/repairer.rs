use std::{fs, path::Path, str::FromStr};

use crate::{
    cli::display::{ask_user, logging::warning, QuestionResponse},
    config::Config,
    installer::{
        types::{PackageId, Version},
        Installer, InstallerOptions, Symlinker,
    },
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    verifier::{error::VerifierError, Issue, Verifier},
};

pub struct Repairer<'a> {
    config: &'a Config,
    manager: &'a RepositoryManager<'a>,
}

impl<'a> Repairer<'a> {
    pub fn new(config: &'a Config, manager: &'a RepositoryManager) -> Self {
        Self { config, manager }
    }

    pub fn fix(&mut self, register: &mut PackageRegister) -> Result<(), VerifierError> {
        let mut verifier = Verifier::new(self.config);
        while let Some(issue) = verifier.next_issue(register)? {
            print!("{issue}\n");

            let question = "Would you like to automatically fix the above issue with `pit fix`?";
            if ask_user(question, QuestionResponse::Yes)?.is_no() {
                continue;
            }

            match issue {
                Issue::BrokenTree(missing) => self.fix_broken_tree(missing, register)?,
                Issue::InconsistentStorage(missing) => self.fix_inconsistent_storage(missing, register)?,
                Issue::InconsistentRegister(missing) => self.fix_inconsistent_register(missing, register)?,
                _ => warning!("Issue fix not yet implemented"),
            }
        }

        Ok(())
    }

    fn fix_broken_tree(&mut self, missing: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) -> Result<(), VerifierError> {
        let installer_options = InstallerOptions::default().skip_symlinking(true);
        let mut installer = Installer::new(&self.config, register, &self.manager, installer_options);

        for (_, missing_package) in missing {
            // TODO: Do we want to check for already existing packages? (there could be duplicates in the missing dependencies)
            installer.install(&missing_package.name, Some(&missing_package.version))?;
        }

        Ok(())
    }

    fn fix_inconsistent_storage(&mut self, missing: Vec<PackageId>, register: &mut PackageRegister) -> Result<(), VerifierError> {
        for missing_package in missing {
            // Gather the package settings before remove the package from the register
            let (symlink, active) = match register.get_package(&missing_package.name) {
                Some(package) => (package.symlinked, package.active_version == missing_package.version),
                None => {
                    warning!("Inconsistent package cannot be found in Installed.toml anymore.");
                    (false, false)
                },
            };

            // Temporarily remove the package from the register
            register.remove_package_version(&missing_package);

            let installer_options = InstallerOptions::default().skip_symlinking(!symlink).skip_active(!active);
            let mut installer = Installer::new(&self.config, register, &self.manager, installer_options);

            installer.install(&missing_package.name, Some(&missing_package.version))?;
        }

        Ok(())
    }

    fn fix_inconsistent_register(&mut self, missing: Vec<PackageId>, register: &mut PackageRegister) -> Result<(), VerifierError> {
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
                Symlinker::new(self.config).remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&package_directory))?;
            }

            // Remove the packages
            fs::remove_dir_all(&package_directory.join(&package_id.name).join(package_id.version.to_string()))?;

            // Re-install the package
            let installer_options = InstallerOptions::default().skip_symlinking(!symlinked).skip_active(!active);
            let mut installer = Installer::new(&self.config, register, &self.manager, installer_options);

            installer.install(&package_id.name, Some(&package_id.version))?;
        }

        Ok(())
    }
}
