// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use std::fmt::Display;

use crate::installer::types::PackageId;

/// Holds a single issue and the data regarding that issue.
pub enum Issue {
    /// A list of parents and their missing dependencies `<parent> : <missing>`.
    BrokenTree(Vec<(PackageId, PackageId)>),

    /// A list of packages which are present in the Installed.toml, but not in the package directory.
    InconsistentStorage(Vec<PackageId>),

    /// A list of packages which are present in the package directory, but not in the Installed.toml.
    InconsistentRegister(Vec<PackageId>),

    /// A list of packages which are changed (when they shouldn't be).
    AlteredPackage(Vec<PackageId>),

    /// The 'packit' group is missing.
    MissingPackitGroup,
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "ISSUE: ".bold().yellow())?;
        match self {
            Issue::BrokenTree(missing) => {
                write!(f, "Broken dependency tree\n")?;
                write!(f, "The following dependencies are missing:\n")?;

                for (parent, missing_package) in missing {
                    let item = format!(
                        "  - {} missing {}\n",
                        parent.to_string().bold().blue(),
                        missing_package.to_string().bold().blue()
                    );
                    write!(f, "{}", item)?;
                }
            },
            Issue::InconsistentStorage(package_ids) if package_ids.len() == 1 => {
                write!(f, "Inconsistent storage\n")?;
                let issue_explanation = format!(
                    "{} was found in Installed.toml, but not in the Packit package directory.\n",
                    package_ids.first().expect("Expected one package id.").to_string().bold().blue()
                );
                write!(f, "{issue_explanation}")?;
            },
            Issue::InconsistentStorage(package_ids) => {
                write!(f, "Inconsistent storage\n")?;
                let issue_explanation = "The following packages were found in Installed.toml, but not in the Packit package directory:\n";
                write!(f, "{issue_explanation}")?;

                for package in package_ids {
                    write!(f, "  - {}\n", package.to_string().bold().blue())?;
                }
            },
            Issue::InconsistentRegister(package_ids) => {
                write!(f, "Inconsistent register\n")?;
                let issue_explanation = "The following packages were found in the Packit packages directory, but not in the register:\n";
                write!(f, "{issue_explanation}")?;

                for package in package_ids {
                    write!(f, "  - {}\n", package.to_string().bold().blue())?;
                }
            },
            Issue::AlteredPackage(altered) => {
                write!(f, "Altered packages\n")?;
                let issue_explanation = "The following packages were found to be changed when they shouldn't be:\n";
                write!(f, "{issue_explanation}")?;

                for package in altered {
                    write!(f, "  - {}\n", package.to_string().bold().blue())?;
                }
            },
            Issue::MissingPackitGroup => {
                write!(f, "Packit group missing\n")?;
                write!(f, "The 'packit' group is missing while multiuser mode is turned on.\n")?;
            },
        }

        Ok(())
    }
}
