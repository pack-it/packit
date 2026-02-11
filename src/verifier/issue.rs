use colored::Colorize;
use std::fmt::Display;

use crate::installer::types::PackageId;

/// This holds a single issue and the data regarding that issue.
pub enum Issue {
    /// A list of parents and their missing dependencies <parent> : <missing>
    BrokenTree(Vec<(PackageId, PackageId)>),

    /// A list of packages which are present in the Installed.toml, but not in the packit package directory
    InconsistentStorage(Vec<PackageId>),
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Do we want to show suggestions on how to fix the issues manually?
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
        }

        Ok(())
    }
}
