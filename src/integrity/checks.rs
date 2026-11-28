use std::sync::LazyLock;

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
    Check::ConfigExistence,
    Check::ConfigSyntax,
    Check::RegisterExistence,
    Check::RegisterSyntax,
    Check::PackitGroup,
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

// TODO: Check if there is a better way to do this
pub static CONFIG_VERIFIED_INDEX: LazyLock<usize> = LazyLock::new(|| {
    let mut verified_index = 0;
    for (index, check) in CHECKS.iter().enumerate() {
        if *check == Check::ConfigExistence || *check == Check::ConfigSyntax {
            verified_index = index;
        }
    }

    verified_index
});

pub static REGISTER_VERIFIED_INDEX: LazyLock<usize> = LazyLock::new(|| {
    let mut verified_index = 0;
    for (index, check) in CHECKS.iter().enumerate() {
        if *check == Check::RegisterExistence || *check == Check::RegisterSyntax {
            verified_index = index;
        }
    }

    verified_index
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
