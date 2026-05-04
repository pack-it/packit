use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Check {
    // Initial checks (checks which verify methods which the verifier uses internally)
    Permissions,
    ConfigExistance,
    ConfigSyntax, // Separate from ConfigExistance, because in the future we implement a different fix (reconstruct from Config.toml)

    // General package checks
    StrayDirectory,
    StorageConsistency,
    RegisterConsistency,
    DependencyTree,
    Alterations,
    PackitGroup,

    // Checks which are specific to a package
    PackageExistance,
    PackageStorageConsistancy,
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
            Self::ConfigExistance => &[Self::Permissions],
            Self::ConfigSyntax => &[Self::Permissions, Self::ConfigExistance],

            // General checks
            Self::PackitGroup => &[],
            Self::StrayDirectory => &[Self::PackitGroup],
            Self::StorageConsistency => &[Self::PackitGroup, Self::StrayDirectory],
            Self::RegisterConsistency => &[Self::PackitGroup, Self::StrayDirectory],
            Self::DependencyTree => &[Self::PackitGroup, Self::StorageConsistency, Self::RegisterConsistency],
            Self::Alterations => &[Self::PackitGroup, Self::StorageConsistency, Self::RegisterConsistency],

            // Package specific checks
            Self::PackageExistance => &[],
            Self::PackageStorageConsistancy => &[Self::PackageExistance],
            Self::PackageRegisterConsistency => &[Self::PackageExistance],
            Self::PackageDependencyTree => &[
                Self::PackageExistance,
                Self::PackageStorageConsistancy,
                Self::PackageRegisterConsistency,
            ],
            Self::PackageAlterations => &[
                Self::PackageExistance,
                Self::PackageStorageConsistancy,
                Self::PackageRegisterConsistency,
            ],
        }
    }

    /// Gets all intial checks.
    pub fn get_initial_checks<'a>() -> &'a [Self] {
        &[Self::Permissions, Self::ConfigExistance, Self::ConfigSyntax]
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
            Self::PackageExistance,
            Self::PackageStorageConsistancy,
            Self::PackageRegisterConsistency,
            Self::PackageDependencyTree,
            Self::PackageAlterations,
        ]
    }

    /// Gets the checks in the correct order based on the 'check dependency tree'.
    /// Returns a flattened 'check dependency tree'
    pub fn get_ordered_checks<'a>(checks: &'a [Self]) -> Vec<&'a Self> {
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
        assert_eq!(
            Check::get_ordered_checks(&[Check::ConfigExistance]),
            Vec::from([&Check::ConfigExistance])
        );

        assert_eq!(
            Check::get_ordered_checks(&[Check::PackageAlterations, Check::PackageRegisterConsistency]),
            Vec::from([
                &Check::PackageExistance,
                &Check::PackageStorageConsistancy,
                &Check::PackageRegisterConsistency,
                &Check::PackageAlterations
            ])
        );

        assert_eq!(
            Check::get_ordered_checks(&[Check::DependencyTree]),
            Vec::from([
                &Check::PackitGroup,
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
