// SPDX-License-Identifier: GPL-3.0-only
use regex::Regex;
use serde::{Deserialize, Serialize, de};
use std::{fmt::Display, str::FromStr, sync::LazyLock};
use thiserror::Error;

const VALID_LICENSE: &str = r"^[a-zA-Z0-9\-\+\.]+$";
static LICENSE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(VALID_LICENSE).expect("Expected valid regex"));

/// Errors that occur when creating or parsing the license identifier.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("Invalid license identifier '{0}', cannot be empty and can only contain characters: 'a-z', 'A-Z', '0-9', '-', '+' and '.'")]
    InvalidLicense(String),
}

/// Represents a license identifier.
#[derive(Serialize, Debug, Clone)]
pub struct LicenseIdentifier(pub String);

impl<'de> Deserialize<'de> for LicenseIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = de::Deserialize::deserialize(deserializer)?;

        Self::from_str(&string).map_err(de::Error::custom)
    }
}

impl FromStr for LicenseIdentifier {
    type Err = LicenseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if !LICENSE_REGEX.is_match(string) {
            return Err(LicenseError::InvalidLicense(string.to_string()));
        }

        Ok(Self(string.to_string()))
    }
}

impl Display for LicenseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl PartialEq for LicenseIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

/// Wrapper to differentiate between different License types in metadata files.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum Licenses {
    #[default]
    Unknown,
    Single(LicenseIdentifier),
    SingleWithExceptions {
        name: LicenseIdentifier,

        #[serde(rename = "with")]
        exceptions: Vec<LicenseIdentifier>,
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
                let exceptions_str = exceptions.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ");
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
    fn valid_from_str() {
        let identifier = &"Test-1.1+";
        assert_eq!(
            LicenseIdentifier::from_str(identifier),
            Ok(LicenseIdentifier(identifier.to_string()))
        );
    }

    #[test]
    fn from_str_no_input() {
        assert_eq!(LicenseIdentifier::from_str(""), Err(LicenseError::InvalidLicense("".to_string())));
    }

    #[test]
    fn from_str_illegal_chars() {
        let illegal_chars = " _/\\!@#$%^&*():;'\"<>[]{}?|~`±§=\u{1234}";
        for identifier in illegal_chars.chars() {
            assert_eq!(
                LicenseIdentifier::from_str(&identifier.to_string()),
                Err(LicenseError::InvalidLicense(identifier.to_string())),
            );
        }
    }

    #[test]
    fn unknown_check() {
        let license = Licenses::Unknown;
        assert!(license.is_unknown());

        let license = Licenses::Single(LicenseIdentifier("MIT".into()));
        assert!(!license.is_unknown());
    }

    #[test]
    fn length() {
        assert_eq!(Licenses::Unknown.get_length(), 0);
        assert_eq!(Licenses::Single(LicenseIdentifier("MIT".into())).get_length(), 1);

        let license = Licenses::SingleWithExceptions {
            name: LicenseIdentifier("MIT".into()),
            exceptions: vec![LicenseIdentifier("Test-Exception".into())],
        };
        assert_eq!(license.get_length(), 1);

        let license = Licenses::Single(LicenseIdentifier("Test".into()));
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
        let license = Licenses::Single(LicenseIdentifier("MIT".into()));
        assert_eq!(license.to_string(), "MIT".to_string());
    }

    #[test]
    fn format_single_with_single_exception() {
        let license = Licenses::SingleWithExceptions {
            name: LicenseIdentifier("MIT".into()),
            exceptions: vec![LicenseIdentifier("Test-Exception".into())],
        };
        assert_eq!(license.to_string(), "MIT WITH Test-Exception".to_string());
    }

    #[test]
    fn format_single_with_exceptions() {
        let license = Licenses::SingleWithExceptions {
            name: LicenseIdentifier("MIT".into()),
            exceptions: vec![
                LicenseIdentifier("Test-Exception".into()),
                LicenseIdentifier("Second-Exception".into()),
            ],
        };
        assert_eq!(license.to_string(), "MIT WITH (Test-Exception, Second-Exception)".to_string());
    }

    #[test]
    fn format_single_with_empty_exceptions() {
        let license = Licenses::SingleWithExceptions {
            name: LicenseIdentifier("MIT".into()),
            exceptions: Vec::new(),
        };
        assert_eq!(license.to_string(), "MIT".to_string());
    }

    #[test]
    fn format_recursive() {
        let license = Licenses::Single(LicenseIdentifier("Test".into()));
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
