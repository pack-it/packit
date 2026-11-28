// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    cli::{
        display::logging::{debug, warning},
        parameter_checks,
    },
    config::Config,
    installer::types::{OptionalPackageId, PackageId},
    integrity::{
        Issue,
        checks::{CHECKS, CONFIG_VERIFIED_INDEX, Check, REGISTER_VERIFIED_INDEX},
        error::{Result, VerifierError},
        repairer, verifier_checks,
    },
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
};

/// Verifier that scans the Packit environment for issues.
pub struct Verifier {
    optional_ids: Vec<OptionalPackageId>,
    expanded_ids: Vec<PackageId>,
    current_check: usize,
    issues_found: u32,
}

impl Verifier {
    /// Creates a new verifier.
    pub fn new(optional_ids: Vec<OptionalPackageId>) -> Self {
        Self {
            optional_ids,
            expanded_ids: Vec::new(),
            current_check: 0,
            issues_found: 0,
        }
    }

    /// Gets the next normal check result.
    /// Returns and `Issue` if an issue is found, `None` if no issues are found.
    pub fn next_check(&mut self) -> Result<Option<Issue>> {
        let check = match CHECKS.get(self.current_check) {
            Some(check) => check,
            None => return Ok(None),
        };

        // Increase current issue
        self.current_check += 1;

        debug!("Performing check '{check:?}'");

        let issue = match check {
            Check::Permissions => verifier_checks::initial::check_permissions()?,
            Check::ConfigExistence => verifier_checks::initial::check_config_existence()?,
            Check::ConfigSyntax => verifier_checks::initial::check_config_syntax()?,
            Check::RegisterExistence => verifier_checks::initial::check_register_existence()?,
            Check::RegisterSyntax => verifier_checks::initial::check_register_syntax()?,
            Check::PackitGroup => verifier_checks::general::check_packit_group(&self.get_config()?)?,
            Check::StrayDirectory => verifier_checks::general::check_stray_directories(&self.get_config()?)?,
            Check::InvalidFiles => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::general::check_invalid_files(&packages, &register, &config)?
            },
            Check::RegisterConsistency => {
                let config = self.get_config()?;
                verifier_checks::package::check_register_consistency(&self.get_register(&config)?, &config)?
            },
            Check::StorageConsistency => {
                let (config, _, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_storage_consistency(&packages, &config)?
            },
            Check::LocalMetadataExistence => {
                let (config, _, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_local_metadata_existence(&packages, &config)?
            },
            Check::DependencyTree => {
                let (_, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_dependency_tree(&packages, &register)
            },
            Check::Alterations => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_alterations(&packages, &register, &config)?
            },
            Check::MissingDependencies => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_missing_dependencies(&packages, &register, &config)?
            },
            Check::InvalidDependencies => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_invalid_dependencies(&packages, &register, &config)?
            },
            Check::MissingDependents => {
                let (_, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_missing_dependents(&packages, &register)
            },
            Check::InvalidDependents => {
                let (_, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_invalid_dependents(&packages, &register)
            },
            Check::MissingDependencySymlinks => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_missing_dir_dependencies(&packages, &register, &config)?
            },
            Check::InvalidActive => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_invalid_active(&packages.iter().map(|p| p.name.clone()).collect(), &register, &config)?
            },
            Check::ForbiddenLink => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_forbidden_link(&packages, &register, &config)?
            },
            Check::MissingLink => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_missing_link(&packages, &register, &config)?
            },
            Check::Test => {
                let (config, register, packages) = self.prepare_arguments()?;
                verifier_checks::package::check_test(&packages, &register, &config)?
            },
        };

        if issue.is_some() {
            self.issues_found += 1;
        }

        Ok(issue)
    }

    /// Fixes the given issue by executing the fix for that issue.
    /// Note: The register is not saved after the fix is applied.
    pub fn fix(&mut self, issue: Issue) -> Result<()> {
        match issue {
            Issue::MissingConfig => repairer::initial::fix_missing_config()?,
            Issue::BrokenConfig => repairer::initial::fix_broken_config()?,
            Issue::MissingRegister => repairer::initial::fix_missing_register()?,
            Issue::BrokenRegister => repairer::initial::fix_broken_register()?,
            Issue::IncorrectPermissions(directories) => repairer::initial::fix_unwritable_directories(directories)?,
            Issue::StrayDirectories(strays) => repairer::general::fix_stray_directories(strays)?,
            Issue::InvalidFiles(invalid) => repairer::general::fix_invalid_files(&invalid)?,
            Issue::BrokenTree(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::fix_broken_tree(missing, &mut register, &config, &manager)?
            },
            Issue::InconsistentStorage(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::fix_inconsistent_storage(missing, &mut register, &config, &manager)?
            },
            Issue::InconsistentRegister(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::fix_inconsistent_register(missing, &mut register, &config, &manager)?
            },
            Issue::MissingLocalMetadata(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::fix_missing_local_metadata(missing, &mut register, &config, &manager)?
            },
            Issue::MissingDependencies(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::fix_missing_dependencies(missing, &mut register, &manager)?
            },
            Issue::InvalidDependencies(invalid) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                repairer::package::fix_invalid_dependencies(invalid, &mut register)?
            },
            Issue::MissingDependents(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                repairer::package::fix_missing_dependents(missing, &mut register)
            },
            Issue::InvalidDependents(invalid) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                repairer::package::fix_invalid_dependents(invalid, &mut register)
            },
            Issue::MissingDependencySymlinks(missing) => {
                let config = self.get_config()?;
                repairer::package::fix_missing_dir_dependencies(missing, &config)?
            },
            Issue::InvalidActive(invalid) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                repairer::package::fix_invalid_active(invalid, &mut register, &config)?
            },
            Issue::ForbiddenLink(forbidden) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                repairer::package::fix_forbidden_link(forbidden, &mut register, &config)?
            },
            Issue::MissingLinks(missing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                repairer::package::fix_missing_links(missing, &mut register, &config)?
            },
            Issue::FailedTest(failing) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::try_reinstall(failing, &mut register, &manager, &config)?
            },
            Issue::AlteredPackage(altered) => {
                let config = self.get_config()?;
                let mut register = self.get_register(&config)?;
                let manager = RepositoryManager::new(&config);
                repairer::package::try_reinstall(altered, &mut register, &manager, &config)?
            },
            Issue::MissingPackitGroup => warning!("Fix not executed, because the issue fix is not yet implemented"),
        }

        Ok(())
    }

    fn prepare_arguments(&mut self) -> Result<(Config, PackageRegister, Vec<PackageId>)> {
        let config = self.get_config()?;
        let register = self.get_register(&config)?;
        let packages = self.get_packages(&register, &config);
        return Ok((config, register, packages));
    }

    /// Gets the config if the config has been verified, otherwise returns a `VerifierError::UseBeforeVerify` error.
    fn get_config(&self) -> Result<Config> {
        if self.current_check > *CONFIG_VERIFIED_INDEX {
            return Ok(Config::from(&Config::get_default_path())?);
        }

        return Err(VerifierError::UseBeforeVerify("config".to_string()));
    }

    /// Gets the register if the register has been verified, otherwise returns a `VerifierError::UseBeforeVerify` error.
    fn get_register(&self, config: &Config) -> Result<PackageRegister> {
        if self.current_check > *REGISTER_VERIFIED_INDEX {
            let register_dir = PackageRegister::get_path(&config.prefix_directory);
            return Ok(PackageRegister::from(&register_dir)?);
        }

        return Err(VerifierError::UseBeforeVerify("register".to_string()));
    }

    // TODO: Check if returning a reference is possible
    fn get_packages(&mut self, register: &PackageRegister, config: &Config) -> Vec<PackageId> {
        if self.optional_ids.is_empty() {
            return register.iterate_all().map(|p| p.package_id.clone()).collect();
        }

        if self.expanded_ids.is_empty() {
            self.expanded_ids = parameter_checks::expand_optional_ids(&register, &config, &self.optional_ids);
        };

        self.expanded_ids.clone()
    }

    /// Get the number of issues found.
    pub fn issues_found(&self) -> u32 {
        self.issues_found
    }

    /// Reverses the checks counter by 1. Except if the current is 0.
    pub fn reverse_check(&mut self) {
        if self.current_check > 0 {
            self.current_check -= 1;
        }
    }

    /// Gets the current check index.
    pub fn get_check_index(&self) -> usize {
        self.current_check
    }

    pub fn get_check_length(&self) -> usize {
        CHECKS.len()
    }
}
