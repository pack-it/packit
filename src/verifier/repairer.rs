use crate::{
    cli::display::{ask_user, logging::warning, QuestionResponse},
    config::Config,
    installer::{types::PackageId, Installer, InstallerOptions},
    repositories::manager::RepositoryManager,
    storage::package_register::PackageRegister,
    verifier::{error::VerifierError, Issue},
};

pub struct Repairer<'a> {
    config: &'a Config,
    register: &'a mut PackageRegister,
    manager: &'a RepositoryManager<'a>,
}

impl<'a> Repairer<'a> {
    pub fn new(config: &'a Config, register: &'a mut PackageRegister, manager: &'a RepositoryManager) -> Self {
        Self { config, register, manager }
    }

    pub fn fix(&mut self, issues: Vec<Issue>) -> Result<(), VerifierError> {
        for issue in issues {
            print!("{issue}\n");

            let question = "Would you like to automatically fix the above issue with `pit fix`?";
            if ask_user(question, QuestionResponse::Yes)?.is_no() {
                continue;
            }

            match issue {
                Issue::BrokenTree(missing) => self.fix_broken_tree(missing)?,
                Issue::InconsistentStorage(missing) => self.fix_inconsistent_storage(missing)?,
                _ => {
                    warning!("Issue fix not yet implemented")
                },
            }
        }

        Ok(())
    }

    fn fix_broken_tree(&mut self, missing: Vec<(PackageId, PackageId)>) -> Result<(), VerifierError> {
        let installer_options = InstallerOptions::default().build_source(false).skip_symlinking(true).skip_active(true).keep_build(false);
        let mut installer = Installer::new(&self.config, self.register, &self.manager, installer_options);

        for (_, missing_package) in missing {
            // TODO: Do we want to check for already existing packages? (there could be duplicates in the missing dependencies)
            installer.install(&missing_package.name, Some(&missing_package.version))?;
        }

        Ok(())
    }

    fn fix_inconsistent_storage(&mut self, missing: Vec<PackageId>) -> Result<(), VerifierError> {
        for missing_package in missing {
            // Gather the package settings before remove the package from the register
            let (symlink, active) = match self.register.get_package(&missing_package.name) {
                Some(package) => (package.symlinked, package.active_version == missing_package.version),
                None => {
                    warning!("Inconsistent package cannot be found in Installed.toml anymore.");
                    (false, false)
                },
            };

            // Temporarily remove the package from the register
            self.register.remove_package_version(&missing_package);

            let installer_options =
                InstallerOptions::default().build_source(false).skip_symlinking(symlink).skip_active(!active).keep_build(false);
            let mut installer = Installer::new(&self.config, self.register, &self.manager, installer_options);

            installer.install(&missing_package.name, Some(&missing_package.version))?;
        }

        Ok(())
    }
}
