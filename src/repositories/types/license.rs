// SPDX-License-Identifier: GPL-3.0-only
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Wrapper to differentiate between different License types in metadata files.
#[cfg_attr(test, derive(PartialEq))]
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

    /// Returns the number of licenses inside a license based on type.
    fn get_length(&self) -> usize {
        match self {
            Licenses::Unknown => 0,
            Licenses::Single(_) => 1,
            Licenses::SingleWithExceptions { .. } => 1,
            Licenses::Any { any } => any.len(),
            Licenses::All { all } => all.len(),
        }
    }

    /// Implementation of license display.
    /// Only includes parentheses for `All` and `Any` options when `include_parentheses` is true.
    fn display_impl(&self, f: &mut std::fmt::Formatter<'_>, include_parentheses: bool) -> std::fmt::Result {
        if include_parentheses && self.get_length() > 1 {
            write!(f, "(")?;
        }

        match self {
            Licenses::Unknown => write!(f, "Unknown")?,
            Licenses::Single(license) => write!(f, "{license}")?,
            Licenses::SingleWithExceptions { name, exceptions } => {
                let exceptions_str = exceptions.join(", ");
                match exceptions.len() {
                    0 => write!(f, "{name}")?,
                    1 => write!(f, "{name} WITH {exceptions_str}")?,
                    _ => write!(f, "{name} WITH ({exceptions_str})")?,
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

        if include_parentheses && self.get_length() > 1 {
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

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn unknown_check() {
        let license = Licenses::Unknown;
        assert!(license.is_unknown());

        let license = Licenses::Single("MIT".to_string());
        assert!(!license.is_unknown());
    }

    #[test]
    fn length() {
        assert_eq!(Licenses::Unknown.get_length(), 0);
        assert_eq!(Licenses::Single("MIT".to_string()).get_length(), 1);

        let license = Licenses::SingleWithExceptions {
            name: "MIT".to_string(),
            exceptions: vec!["Test-Exception".to_string()],
        };
        assert_eq!(license.get_length(), 1);

        let license = Licenses::Single("Test".to_string());
        let recursive_license = Licenses::All {
            all: vec![license.clone(), license.clone()],
        };
        assert_eq!(recursive_license.get_length(), 2);

        // Double recursive
        let recursive_license = Licenses::Any {
            any: vec![recursive_license],
        };
        assert_eq!(recursive_license.get_length(), 1);
    }

    #[test]
    fn format_unknown() {
        let license = Licenses::Unknown;
        assert_eq!(license.to_string(), "Unknown");
    }

    #[test]
    fn format_single() {
        let license = Licenses::Single("MIT".to_string());
        assert_eq!(license.to_string(), "MIT".to_string());
    }

    #[test]
    fn format_single_with_single_exception() {
        let license = Licenses::SingleWithExceptions {
            name: "MIT".to_string(),
            exceptions: vec!["Test-Exception".to_string()],
        };
        assert_eq!(license.to_string(), "MIT WITH Test-Exception".to_string());
    }

    #[test]
    fn format_single_with_exceptions() {
        let license = Licenses::SingleWithExceptions {
            name: "MIT".to_string(),
            exceptions: vec!["Test-Exception".to_string(), "Second-Exception".to_string()],
        };
        assert_eq!(license.to_string(), "MIT WITH (Test-Exception, Second-Exception)".to_string());
    }

    #[test]
    fn format_single_with_empty_exceptions() {
        let license = Licenses::SingleWithExceptions {
            name: "MIT".to_string(),
            exceptions: Vec::new(),
        };
        assert_eq!(license.to_string(), "MIT".to_string());
    }

    #[test]
    fn format_recursive() {
        let license = Licenses::Single("Test".to_string());
        let recursive_license = Licenses::All {
            all: vec![license.clone()],
        };
        assert_eq!(recursive_license.to_string(), "Test");

        // Double recursive
        let recursive_license = Licenses::Any {
            any: vec![recursive_license, license],
        };
        assert_eq!(recursive_license.to_string(), "Test OR Test");
    }

    #[test]
    fn format_empty_recursive() {
        let license = Licenses::All { all: Vec::new() };
        assert_eq!(license.to_string(), "");

        let license = Licenses::Any { any: Vec::new() };
        assert_eq!(license.to_string(), "");
    }
}
