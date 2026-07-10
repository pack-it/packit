// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    installer::types::{OptionalPackageId, PackageId, PackageName, Version},
    repositories::types::Licenses,
};
use colored::{ColoredString, Colorize};
use std::fmt::Display;

pub trait Styled: Display {
    /// Styles a value which implements `Display`.
    /// Returns a `ColoredString`.
    fn style(&self) -> ColoredString;
}

// Implements Styled for all references of T: Styled
impl<T: Styled> Styled for &T {
    fn style(&self) -> ColoredString {
        (*self).style()
    }
}

impl Styled for PackageId {
    fn style(&self) -> ColoredString {
        self.to_string().bold().blue()
    }
}

impl Styled for OptionalPackageId {
    fn style(&self) -> ColoredString {
        self.to_string().bold().blue()
    }
}

impl Styled for PackageName {
    fn style(&self) -> ColoredString {
        self.to_string().bold().blue()
    }
}

impl Styled for Version {
    fn style(&self) -> ColoredString {
        self.to_string().bold().blue()
    }
}

impl Styled for Licenses {
    fn style(&self) -> ColoredString {
        match self {
            Licenses::Unknown => self.to_string().dimmed(),
            _ => self.to_string().normal(),
        }
    }
}
