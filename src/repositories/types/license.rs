// SPDX-License-Identifier: GPL-3.0-only
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Wrapper to differentiate between different License types in metadata files.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum Licenses {
    #[default]
    Unknown,
    Single(String),
    SingleWithExceptions {
        name: String,

        #[serde(rename = "with")]
        exceptions: Vec<String>,
    },
    Any {
        any: Vec<Licenses>,
    },
    All {
        all: Vec<Licenses>,
    },
}

impl Licenses {
    /// Returns true if the license is `Unknown`.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Implementation of license display.
    /// Only includes parentheses for `All` and `Any` options when `include_parentheses` is true.
    fn display_impl(&self, f: &mut std::fmt::Formatter<'_>, include_parentheses: bool) -> std::fmt::Result {
        if include_parentheses && matches!(self, Licenses::All { .. } | Licenses::Any { .. }) {
            write!(f, "(")?;
        }

        match self {
            Licenses::Unknown => write!(f, "Unknown")?,
            Licenses::Single(license) => write!(f, "{license}")?,
            Licenses::SingleWithExceptions { name, exceptions } => {
                write!(f, "{name} WITH ",)?;
                let exceptions_str = exceptions.join(", ");
                match exceptions.len() {
                    1 => write!(f, "{}", exceptions_str)?,
                    _ => write!(f, "({})", exceptions_str)?,
                }
            },
            Licenses::Any { any } => {
                for (i, license) in any.iter().enumerate() {
                    if i > 0 {
                        write!(f, " OR ")?;
                    }
                    license.display_impl(f, true)?;
                }
            },
            Licenses::All { all } => {
                for (i, license) in all.iter().enumerate() {
                    if i > 0 {
                        write!(f, " AND ")?;
                    }
                    license.display_impl(f, true)?;
                }
            },
        }

        if include_parentheses && matches!(self, Licenses::All { .. } | Licenses::Any { .. }) {
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl Display for Licenses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, false)
    }
}
