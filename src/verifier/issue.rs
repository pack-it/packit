// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use std::{collections::HashSet, fmt::Display, path::PathBuf};

use crate::installer::types::PackageId;

/// Holds a single issue and the data regarding that issue.
pub enum Issue {
    /// The user cannot write to the prefix directory (or one of its sub directories).
    IncorrectPermissions(HashSet<PathBuf>),

    /// The Packit Config.toml is missing.
    MissingConfig,

    /// The Packit Installed.toml is missing.
    MissingRegister,

    /// A list of parents and their missing dependencies `<parent> : <missing>`.
    BrokenTree(Vec<(PackageId, PackageId)>),

    /// A list of packages which are present in the Installed.toml, but not in the package directory.
    InconsistentStorage(Vec<PackageId>),

    /// A list of packages which are present in the package directory, but not in the Installed.toml.
    InconsistentRegister(HashSet<PackageId>),

    /// A list of packages which are changed (when they shouldn't be).
    AlteredPackage(Vec<PackageId>),

    /// The 'packit' group is missing.
    MissingPackitGroup,

    /// The given package cannot be found anywhere. This issue only applies when a package is specified by the user.
    NotFound(PackageId),

    /// A list of stray directories.
    StrayDirectories(HashSet<PathBuf>),
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "ISSUE: ".bold().yellow())?;
        match self {
            Issue::IncorrectPermissions(directories) => {
                writeln!(f, "Incorrect permissions")?;
                let issue_explanation = "The following directories were not writable:";
                writeln!(f, "{issue_explanation}")?;

                for directory in directories {
                    writeln!(f, "  - {}", directory.display())?;
                }
            },
            Issue::MissingConfig => {
                writeln!(f, "Missing Config.toml file")?;
            },
            Issue::MissingRegister => {
                writeln!(f, "Missing Installed.toml file")?;
            },
            Issue::BrokenTree(missing) => {
                writeln!(f, "Broken dependency tree")?;
                writeln!(f, "The following dependencies are missing:")?;

                for (parent, missing_package) in missing {
                    let item = format!(
                        "  - {} missing {}",
                        parent.to_string().bold().blue(),
                        missing_package.to_string().bold().blue()
                    );

                    writeln!(f, "{}", item)?;
                }
            },
            Issue::InconsistentStorage(package_ids) => {
                writeln!(f, "Inconsistent storage")?;
                let issue_explanation = "The following packages were found in Installed.toml, but not in the Packit package directory:";
                writeln!(f, "{issue_explanation}")?;

                for package in package_ids {
                    writeln!(f, "  - {}", package.to_string().bold().blue())?;
                }
            },
            Issue::InconsistentRegister(package_ids) => {
                writeln!(f, "Inconsistent register")?;
                let issue_explanation = "The following packages were found in the Packit packages directory, but not in the register:";
                writeln!(f, "{issue_explanation}")?;

                for package in package_ids {
                    writeln!(f, "  - {}", package.to_string().bold().blue())?;
                }
            },
            Issue::AlteredPackage(altered) => {
                writeln!(f, "Altered packages")?;
                let issue_explanation = "The following packages were found to be changed when they shouldn't be:";
                writeln!(f, "{issue_explanation}")?;

                for package in altered {
                    writeln!(f, "  - {}", package.to_string().bold().blue())?;
                }
            },
            Issue::MissingPackitGroup => {
                writeln!(f, "Packit group missing")?;
                writeln!(f, "The 'packit' group is missing while multiuser mode is turned on.")?;
            },
            Issue::NotFound(package_id) => {
                writeln!(f, "Package existence")?;
                writeln!(f, "{} cannot be found anywhere in Packit.", package_id.to_string().bold().blue())?

                // TODO: Somehow show result of fuzzy search here
            },
            Issue::StrayDirectories(directories) => {
                writeln!(f, "Stray directories")?;
                let issue_explanation = "The following stray directories were found inside of prefix/packages:";
                writeln!(f, "{issue_explanation}")?;

                for directory in directories {
                    writeln!(f, "  - {}", directory.display())?;
                }
            },
        }

        Ok(())
    }
}

impl Issue {
    /// Gets a message which descripes the fix for each issue.
    pub fn get_fix_message(&self) -> &str {
        match &self {
            Issue::IncorrectPermissions(_) => "To fix this issue we set the permissions again.",
            Issue::MissingConfig => "To fix this issue we try to reconstruct the Config.toml with data still in the Packit directory.",
            Issue::MissingRegister => "To fix this issue we try to reconstruct the Installed.toml with data still in the Packit directory.",
            Issue::StrayDirectories(_) => "To fix this issue we remove the stray directories.",
            Issue::BrokenTree(_) => "To fix this issue the missing packages will be installed.",
            Issue::InconsistentStorage(_) => {
                "To fix this issue the packages are temporarily removed from the register and then reinstalled."
            },
            Issue::InconsistentRegister(_) => "To fix this issue the packages are temporarily removed from storage and then reinstalled.",
            Issue::AlteredPackage(_) | Issue::MissingPackitGroup | Issue::NotFound(_) => {
                "There is no automatic fix for this issue available yet."
            },
        }
    }
}
