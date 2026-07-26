// SPDX-License-Identifier: GPL-3.0-only
use std::{error::Error, fmt::Display};

use colored::Colorize;

use crate::cli::display::logging::error;

/// Represents an issue found with the metadata checks.
pub struct MetaIssue {
    pub issue_type: IssueType,
    pub description: String,
    pub checks_skipped: bool,
    pub error: Option<Box<dyn Error>>,
    pub suggestion: Option<String>,
}

impl MetaIssue {
    /// Creates a default `MetaIssue`.
    pub fn default(description: String) -> Self {
        Self {
            issue_type: IssueType::Breaking,
            description,
            checks_skipped: false,
            error: None,
            suggestion: None,
        }
    }

    /// Set the issue type.
    pub fn set_issue_type(mut self, issue_type: IssueType) -> Self {
        self.issue_type = issue_type;
        self
    }

    /// Set the checks skipped value.
    pub fn set_checks_skipped(mut self, checks_skipped: bool) -> Self {
        self.checks_skipped = checks_skipped;
        self
    }

    /// Set the error. Note that the error argument can only be `Some`.
    pub fn set_error(mut self, error: Box<dyn Error>) -> Self {
        self.error = Some(error);
        self
    }

    /// Set a suggestion for the issue.
    pub fn set_suggestion(mut self, suggestion: Option<String>) -> Self {
        self.suggestion = suggestion;
        self
    }
}

impl Display for MetaIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.issue_type {
            IssueType::Fatal => &"FATAL".bold().red(),
            IssueType::Breaking => &"BREAKING".bold().red(),
            IssueType::Warning => &"WARNING".bold().yellow(),
        };

        writeln!(f, "{}: {}", prefix, self.description)?;

        if let Some(suggestion) = &&self.suggestion {
            writeln!(f, " \u{2514}\u{2500}> Maybe try this suggestion: {suggestion}")?;
        }

        if let Some(error) = &self.error {
            error!(&**error);
        }

        if self.checks_skipped {
            writeln!(
                f,
                "Due to the issue above other checks regarding this item were not possible, so some checks were skipped"
            )?;
        }

        Ok(())
    }
}

/// Represents different types of issues.
pub enum IssueType {
    // Cannot continue with checks
    Fatal,

    // The metadata contains information which breaks certain logic, but continueing is possible
    Breaking,

    // The metadata is correct and functions, however it's unconventional (although maybe unavoidable)
    Warning,
}
