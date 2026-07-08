use crate::installer::types::{OptionalPackageId, PackageId, PackageName, Version};
use colored::{ColoredString, Colorize};
use std::fmt::Display;

pub trait Styled: Display {
    fn style(&self) -> ColoredString;
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
