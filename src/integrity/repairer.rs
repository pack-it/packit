// SPDX-License-Identifier: GPL-3.0-only
mod general;
mod initial;
mod package;

use crate::{
    cli::display::logging::warning,
    config::Config,
    integrity::{Issue, error::Result},
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
};

/// Repairer that fixes issues found by the verifier.
pub struct Repairer;

impl Repairer {
    /// Creates a new repairer.
    pub fn new() -> Self {
        Self
    }

    /// Fixes the initial issues (issues which are critical for Packit itself to work).
    pub fn fix_initial_issues(&mut self, issue: Issue) -> Result<()> {
        match issue {
            Issue::MissingConfig => initial::fix_missing_config()?,
            Issue::IncorrectPermissions(directories) => initial::fix_unwritable_directories(directories)?,
            Issue::MissingRegister => initial::fix_missing_register()?,

            _ => warning!("Fix not executed, because it is not an initial issue"),
        }

        Ok(())
    }

    /// Fixes the given issue by executing the fix for that issue.
    /// Note: The register is not saved after the fix is applied.
    pub fn fix(&mut self, issue: Issue, register: &mut PackageRegister, config: &Config, manager: &RepositoryManager) -> Result<()> {
        match issue {
            Issue::BrokenTree(missing) => package::fix_broken_tree(missing, register, config, manager)?,
            Issue::InconsistentStorage(missing) => package::fix_inconsistent_storage(missing, register, config, manager)?,
            Issue::InconsistentRegister(missing) => package::fix_inconsistent_register(missing, register, config, manager)?,
            Issue::StrayDirectories(strays) => general::fix_stray_directories(strays)?,
            Issue::MissingDependencies(missing) => package::fix_missing_dependencies(missing, register, manager)?,
            Issue::InvalidDependencies(invalid) => package::fix_invalid_dependencies(invalid, register)?,
            Issue::MissingDependents(missing) => package::fix_missing_dependents(missing, register),
            Issue::InvalidDependents(invalid) => package::fix_invalid_dependents(invalid, register),
            Issue::InvalidActive(invalid) => package::fix_invalid_active(invalid, register, config)?,
            Issue::ForbiddenLink(forbidden) => package::fix_forbidden_link(forbidden, register, config)?,
            Issue::MissingLinks(missing) => package::fix_missing_links(missing, register, config)?,
            Issue::FailedTest(failing) => package::try_reinstall(failing, register, manager, config)?,
            Issue::AlteredPackage(altered) => package::try_reinstall(altered, register, manager, config)?,
            _ => warning!("Fix not executed, because the issue fix is not yet implemented"),
        }

        Ok(())
    }
}
