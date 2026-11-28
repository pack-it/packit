// SPDX-License-Identifier: GPL-3.0-only
use std::sync::LazyLock;

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
    Check::PackitGroup, // Done last, because there is no fix yet
];

// Get the index of the last config check (after this check the config can be used)
pub static CONFIG_VERIFIED_INDEX: LazyLock<usize> = LazyLock::new(|| {
    CHECKS
        .iter()
        .rposition(|check| *check == Check::ConfigExistence || *check == Check::ConfigSyntax)
        .expect("Expected config check to exist")
});

// Get the index of the last register check (after this check the register can be used)
pub static REGISTER_VERIFIED_INDEX: LazyLock<usize> = LazyLock::new(|| {
    CHECKS
        .iter()
        .rposition(|check| *check == Check::RegisterExistence || *check == Check::RegisterSyntax)
        .expect("Expected register check to exist")
});

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

    #[test]
    fn check_index_order() {
        assert!(*CONFIG_VERIFIED_INDEX < *REGISTER_VERIFIED_INDEX);
    }
}
