use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Check {
    // Initial checks (checks which verify methods which the verifier uses internally)
    Permissions,
    ConfigExistence,
    ConfigSyntax, // Separate from ConfigExistence, because in the future we implement a different fix (reconstruct from Config.toml)
    RegisterExistence,
    RegisterSyntax, // Separate from RegisterExistence, same reason as for ConfigSyntax

    // General package checks
    StrayDirectory,
    StorageConsistency,
    RegisterConsistency,
    DependencyTree,
    Alterations,
    PackitGroup,

    // Checks which are specific to a package
    PackageExistence,
    PackageStorageConsistency,
    PackageRegisterConsistency,
    PackageDependencyTree,
    PackageAlterations,
}

impl Check {
    /// Gets the dependencies of a check (the checks which should happen before the given check).
    fn get_dependencies(&self) -> &[Self] {
        match self {
            // Initial checks
            Self::Permissions => &[],
            Self::ConfigExistence => &[Self::Permissions],
            Self::ConfigSyntax => &[Self::Permissions, Self::ConfigExistence],
            Self::RegisterExistence => &[Self::Permissions, Self::ConfigExistence, Self::ConfigSyntax],
            Self::RegisterSyntax => &[
                Self::Permissions,
                Self::ConfigExistence,
                Self::ConfigSyntax,
                Self::RegisterExistence,
            ],

            // General checks
            Self::PackitGroup => &[],
            Self::StrayDirectory => &[Self::PackitGroup],
            Self::StorageConsistency => &[Self::PackitGroup, Self::StrayDirectory],
            Self::RegisterConsistency => &[Self::PackitGroup, Self::StrayDirectory],
            Self::DependencyTree => &[Self::PackitGroup, Self::StorageConsistency, Self::RegisterConsistency],
            Self::Alterations => &[Self::PackitGroup, Self::StorageConsistency, Self::RegisterConsistency],

            // Package specific checks
            Self::PackageExistence => &[],
            Self::PackageStorageConsistency => &[Self::PackageExistence],
            Self::PackageRegisterConsistency => &[Self::PackageExistence],
            Self::PackageDependencyTree => &[
                Self::PackageExistence,
                Self::PackageStorageConsistency,
                Self::PackageRegisterConsistency,
            ],
            Self::PackageAlterations => &[
                Self::PackageExistence,
                Self::PackageStorageConsistency,
                Self::PackageRegisterConsistency,
            ],
        }
    }

    /// Gets all intial checks.
    pub fn get_initial_checks<'a>() -> &'a [Self] {
        &[
            Self::Permissions,
            Self::ConfigExistence,
            Self::ConfigSyntax,
            Self::RegisterExistence,
            Self::RegisterSyntax,
        ]
    }

    /// Gets all general checks.
    pub fn get_general_checks<'a>() -> &'a [Self] {
        &[
            Self::StorageConsistency,
            Self::RegisterConsistency,
            Self::DependencyTree,
            Self::Alterations,
            Self::PackitGroup,
            Self::StrayDirectory,
        ]
    }

    /// Gets all package specific checks.
    pub fn get_package_checks<'a>() -> &'a [Self] {
        &[
            Self::PackageExistence,
            Self::PackageStorageConsistency,
            Self::PackageRegisterConsistency,
            Self::PackageDependencyTree,
            Self::PackageAlterations,
        ]
    }

    /// Gets the checks in the correct order based on the 'check dependency tree'.
    /// Returns a flattened 'check dependency tree'
    pub fn get_ordered_checks(checks: &[Self]) -> Vec<&Self> {
        let mut ordered = Vec::new();
        for check in checks {
            ordered.extend(Self::get_ordered_checks(check.get_dependencies()));
            ordered.push(check);
        }

        let mut seen = HashSet::new();
        let mut unique_ordered = Vec::new();
        for check in ordered {
            if !seen.contains(check) {
                unique_ordered.push(check);
                seen.insert(check);
            }
        }

        unique_ordered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_order() {
        assert_eq!(Check::get_ordered_checks(&[Check::Permissions]), Vec::from([&Check::Permissions]));

        assert_eq!(
            Check::get_ordered_checks(&[Check::PackageAlterations, Check::PackageRegisterConsistency]),
            Vec::from([
                &Check::PackageExistence,
                &Check::PackageStorageConsistency,
                &Check::PackageRegisterConsistency,
                &Check::PackageAlterations
            ])
        );

        assert_eq!(
            Check::get_ordered_checks(&[Check::DependencyTree]),
            Vec::from([
                &Check::PackitGroup,
                &Check::StrayDirectory,
                &Check::StorageConsistency,
                &Check::RegisterConsistency,
                &Check::DependencyTree,
            ])
        );
    }

    #[test]
    fn check_cycles() {
        let mut all_checks = Check::get_initial_checks().to_vec();
        all_checks.extend(Check::get_general_checks().to_vec());
        all_checks.extend(Check::get_package_checks().to_vec());

        for check in all_checks {
            check_cylces_impl(&check.clone(), &mut HashSet::from([check]));
        }
    }

    fn check_cylces_impl(parent: &Check, seen: &mut HashSet<Check>) {
        for check in parent.get_dependencies() {
            if seen.contains(check) {
                panic!("Cycle found in verifier checks")
            }

            seen.insert(check.clone());
            check_cylces_impl(check, seen);
            seen.remove(check);
        }
    }
}
