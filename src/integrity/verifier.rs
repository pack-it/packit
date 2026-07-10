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
    current_intial_check: usize,
    current_check: usize,
    issues_found: bool,
}

impl Verifier {
    /// Creates a new verifier.
    pub fn new() -> Self {
        Self {
            current_intial_check: 0,
            current_check: 0,
            issues_found: false,
        }
    }

    /// Gets the result of the next initial check.
    /// If an error occurs during the check it's only returned if no previous issues were found.
    pub fn next_initial_check(&mut self) -> Result<Option<Issue>> {
        match self.next_initial_check_impl() {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining checks.");
                self.current_intial_check = Check::get_initial_checks().len();
                Ok(None)
            },
            Err(e) => Err(e),
        }
    }

    /// Gets the result of the next initial check.
    /// Returns and `Issue` if an issue is found, `None` if no issues are found.
    fn next_initial_check_impl(&mut self) -> Result<Option<Issue>> {
        let ordered_checks = Check::get_ordered_checks(Check::get_initial_checks());
        let check = match ordered_checks.get(self.current_intial_check) {
            Some(check) => check,
            None => return Ok(None),
        };

        // Increase current issue
        self.current_intial_check += 1;

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
            self.issues_found = true;
        }

        return Ok(issue);
    }

    /// Gets the next normal check result.
    /// If an error occurs during the check it's only returned if no previous issues were found.
    pub fn next_check(&mut self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        // Make sure the initial checks have been run before doing general checks
        if self.current_intial_check != Check::get_initial_checks().len() {
            return Err(VerifierError::InitialChecksSkipped);
        }

        match self.next_check_impl(packages, register, config) {
            Ok(issue) => Ok(issue),
            Err(e) if self.issues_found => {
                debug!(err: e, "An error occured when issues were already found, skipping remaining checks.");
                self.current_check = Check::get_checks().len();
                Ok(None)
            },
            Err(e) => Err(e),
        }
    }

    /// Gets the next normal check result.
    /// Returns and `Issue` if an issue is found, `None` if no issues are found.
    fn next_check_impl(&mut self, packages: &Vec<PackageId>, register: &PackageRegister, config: &Config) -> Result<Option<Issue>> {
        let ordered_checks = Check::get_ordered_checks(Check::get_checks());
        let check = match ordered_checks.get(self.current_check) {
            Some(check) => check,
            None => return Ok(None),
        };

        // Increase current issue
        self.current_check += 1;

        let issue = match check {
            Check::PackitGroup => general::check_packit_group(config)?,
            Check::StrayDirectory => general::check_stray_directories(config)?,
            Check::RegisterConsistency => package::check_register_consistency(register, config)?,
            Check::StorageConsistency => package::check_storage_consistency(packages, config)?,
            Check::DependencyTree => package::check_dependency_tree(packages, register),
            Check::Alterations => package::check_alterations(packages, register, config)?,
            Check::MissingDependencies => package::check_missing_dependencies(packages, register)?,
            Check::InvalidDependencies => package::check_invalid_dependencies(packages, register)?,
            Check::MissingDependents => package::check_missing_dependents(packages, register),
            Check::InvalidDependents => package::check_invalid_dependents(packages, register),
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
            self.issues_found = true;
        }

        return Ok(issue);
    }

    /// Get the issues found state.
    /// Returns true if issues are found, false otherwise.
    pub fn issues_found(&self) -> bool {
        self.issues_found
    }

    /// Reverses the initial checks counter by 1. Except if the current is 0.
    /// Returns the new value of current_intial_check.
    pub fn reverse_initial_check(&mut self) -> usize {
        if self.current_intial_check > 0 {
            self.current_intial_check -= 1;
        }

        self.current_intial_check
    }

    /// Reverses the checks counter by 1. Except if the current is 0.
    /// Returns the new value of `current_check`.
    pub fn reverse_check(&mut self) -> usize {
        if self.current_check > 0 {
            self.current_check -= 1;
        }

        self.current_check
    }

    /// Gets the current initial check index.
    pub fn get_initial_check_index(&self) -> usize {
        self.current_intial_check
    }

    /// Gets the current check index.
    pub fn get_check_index(&self) -> usize {
        self.current_check
    }

    /// Gets the number of initial checks.
    pub fn get_initial_check_length(&self) -> usize {
        Check::get_initial_checks().len()
    }

    /// Gets the number of normal checks.
    pub fn get_check_length(&self) -> usize {
        Check::get_checks().len()
    }
}
