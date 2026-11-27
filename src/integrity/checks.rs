// SPDX-License-Identifier: GPL-3.0-only
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Check {
    Permissions,
    ConfigExistence,
    ConfigSyntax,
    RegisterExistence,
    RegisterSyntax,
    StrayDirectory,
    PackitGroup,
    StorageConsistency,
    RegisterConsistency,
    LocalMetadataExistence,
    DependencyTree,
    Alterations,
    MissingDependents,
    InvalidDependents,
    InvalidActive,
    ForbiddenLink,
    MissingLink,
    MissingDependencies,
    InvalidDependencies,
    MissingDependencySymlinks,
    InvalidFiles,
    Test,
}

// The order in which all checks should happen.
pub const CHECKS: [Check; 22] = [
    Check::Permissions,
    Check::PackitGroup,
    Check::ConfigExistence,
    Check::ConfigSyntax,
    Check::RegisterExistence,
    Check::RegisterSyntax,
    Check::StrayDirectory,
    Check::ForbiddenLink,
    Check::MissingLink,
    Check::StorageConsistency,
    Check::RegisterConsistency,
    Check::LocalMetadataExistence,
    Check::InvalidActive,
    Check::MissingDependencies,
    Check::InvalidDependencies,
    Check::MissingDependents,
    Check::InvalidDependents,
    Check::DependencyTree,
    Check::MissingDependencySymlinks,
    Check::InvalidFiles,
    Check::Alterations,
    Check::Test,
];

impl Check {
    /// Gets all initial checks.
    pub const fn get_initial_checks<'a>() -> &'a [Self] {
        &[
            Self::Permissions,
            Self::ConfigExistence,
            Self::ConfigSyntax,
            Self::RegisterExistence,
            Self::RegisterSyntax,
        ]
    }

    /// Gets the normal checks.
    pub const fn get_checks<'a>() -> &'a [Self] {
        &[
            Self::StorageConsistency,
            Self::RegisterConsistency,
            Self::LocalMetadataExistence,
            Self::DependencyTree,
            Self::Alterations,
            Self::PackitGroup,
            Self::StrayDirectory,
            Self::MissingDependents,
            Self::InvalidDependents,
            Self::InvalidActive,
            Self::ForbiddenLink,
            Self::MissingLink,
            Self::MissingDependencies,
            Self::InvalidDependencies,
            Self::MissingDependencySymlinks,
            Self::InvalidFiles,
            Self::Test,
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn check_duplicates() {
        let mut seen = HashSet::new();
        for check in CHECKS {
            assert!(!seen.contains(&check));
            seen.insert(check);
        }
    }
}
