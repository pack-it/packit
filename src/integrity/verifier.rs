// SPDX-License-Identifier: GPL-3.0-only
mod general;
mod initial;
mod package;

use crate::{
    cli::display::logging::debug,
    config::Config,
    installer::types::PackageId,
    integrity::{
        Issue,
        checks::Check,
        error::{Result, VerifierError},
    },
    register::package_register::PackageRegister,
};

/// Verifier that scans the Packit environment for issues.
pub struct Verifier {
    current_initial_check: usize,
    current_check: usize,
    issues_found: u32,
}

impl Verifier {
    /// Creates a new verifier.
    pub fn new() -> Self {
        Self {
            current_initial_check: 0,
            current_check: 0,
            issues_found: 0,
        }
    }

    /// Gets the result of the next initial check.
    /// Returns and `Issue` if an issue is found, `None` if no issues are found.
    pub fn next_initial_check(&mut self) -> Result<Option<Issue>> {
        let ordered_checks = Check::get_ordered_checks(Check::get_initial_checks());
        let check = match ordered_checks.get(self.current_initial_check) {
            Some(check) => check,
            None => return Ok(None),
        };

        // Increase current issue
        self.current_initial_check += 1;

        debug!("Performing check '{check:?}'");

        let issue = match check {
            Check::Permissions => initial::check_permissions()?,
            Check::ConfigExistence => initial::check_config_existence()?,
            Check::ConfigSyntax => initial::check_config_syntax()?,
            Check::RegisterExistence => initial::check_register_existence()?,
            Check::RegisterSyntax => initial::check_register_syntax()?,

            // Return `VerifierError::UnimplementedCheck` if the current check is an initial check (meaning it's not implemented).
            _ if Check::get_initial_checks().contains(check) => return Err(VerifierError::UnimplementedCheck),

            // Return early if the check is not an initial check
            _ => return Ok(None),
        };

        if issue.is_some() {
            self.issues_found += 1;
        }

        Ok(issue)
    }

    /// Gets the next normal check result.
    /// Returns and `Issue` if an issue is found, `None` if no issues are found.
    pub fn next_check(&mut self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        // Make sure the initial checks have been run before doing general checks
        if self.current_initial_check != Check::get_initial_checks().len() {
            return Err(VerifierError::InitialChecksSkipped);
        }

        let ordered_checks = Check::get_ordered_checks(Check::get_checks());
        let check = match ordered_checks.get(self.current_check) {
            Some(check) => check,
            None => return Ok(None),
        };

        // Increase current issue
        self.current_check += 1;

        debug!("Performing check '{check:?}'");

        let issue = match check {
            Check::PackitGroup => general::check_packit_group(config)?,
            Check::StrayDirectory => general::check_stray_directories(config)?,
            Check::InvalidFiles => general::check_invalid_files(packages, register, config)?,
            Check::RegisterConsistency => package::check_register_consistency(register, config)?,
            Check::StorageConsistency => package::check_storage_consistency(packages, config)?,
            Check::DependencyTree => package::check_dependency_tree(packages, register),
            Check::Alterations => package::check_alterations(packages, register, config)?,
            Check::MissingDependencies => package::check_missing_dependencies(packages, register)?,
            Check::InvalidDependencies => package::check_invalid_dependencies(packages, register)?,
            Check::MissingDependents => package::check_missing_dependents(packages, register),
            Check::InvalidDependents => package::check_invalid_dependents(packages, register),
            Check::MissingDependencySymlinks => package::check_missing_dir_dependencies(packages, register, config)?,
            Check::InvalidActive => package::check_invalid_active(&packages.iter().map(|p| p.name.clone()).collect(), register, config)?,
            Check::ForbiddenLink => package::check_forbidden_link(packages, register)?,
            Check::MissingLink => package::check_missing_link(packages, register, config)?,
            Check::Test => package::check_test(packages, register, config)?,

            // Return `VerifierError::UnimplementedCheck` if the current check is a general check (meaning it's not implemented).
            _ if Check::get_checks().contains(check) => return Err(VerifierError::UnimplementedCheck),

            // Return early if the check is an initial check
            _ => return Ok(None),
        };

        if issue.is_some() {
            self.issues_found += 1;
        }

        Ok(issue)
    }

    /// Get the number of issues found.
    pub fn issues_found(&self) -> u32 {
        self.issues_found
    }

    /// Reverses the initial checks counter by 1. Except if the current is 0.
    pub fn reverse_initial_check(&mut self) {
        if self.current_initial_check > 0 {
            self.current_initial_check -= 1;
        }
    }

    /// Reverses the checks counter by 1. Except if the current is 0.
    pub fn reverse_check(&mut self) {
        if self.current_check > 0 {
            self.current_check -= 1;
        }
    }

    /// Gets the current initial check index.
    pub fn get_initial_check_index(&self) -> usize {
        self.current_initial_check
    }

    /// Gets the current check index.
    pub fn get_check_index(&self) -> usize {
        self.current_check
    }

    /// Gets the number of initial checks.
    pub fn get_initial_check_length(&self) -> usize {
        Check::get_ordered_checks(Check::get_initial_checks()).len()
    }

    /// Gets the number of normal checks.
    pub fn get_check_length(&self) -> usize {
        Check::get_ordered_checks(Check::get_checks()).len()
    }
}
