// SPDX-License-Identifier: GPL-3.0-only

//! This module contains custom parser implementations for clap.
//!
//! Currently they are only used to allow uppercase characters in package names.
//! The parsers automatically translate these to lowercase names.
use std::ffi::OsStr;

use clap::{
    Arg, Command,
    builder::{TypedValueParser, ValueParserFactory},
};

use crate::installer::types::{OptionalPackageId, PackageId, PackageName};

#[derive(Clone, Debug)]
pub struct PackageNameParser;

impl TypedValueParser for PackageNameParser {
    type Value = PackageName;

    fn parse_ref(&self, cmd: &Command, arg: Option<&Arg>, value: &OsStr) -> Result<Self::Value, clap::error::Error> {
        let inner_parser = <Self::Value as std::str::FromStr>::from_str;
        inner_parser.parse_ref(cmd, arg, &value.to_ascii_lowercase())
    }
}

impl ValueParserFactory for PackageName {
    type Parser = PackageNameParser;

    fn value_parser() -> Self::Parser {
        PackageNameParser
    }
}

#[derive(Clone, Debug)]
pub struct PackageIdParser;

impl TypedValueParser for PackageIdParser {
    type Value = PackageId;

    fn parse_ref(&self, cmd: &Command, arg: Option<&Arg>, value: &OsStr) -> Result<Self::Value, clap::error::Error> {
        let inner_parser = <Self::Value as std::str::FromStr>::from_str;
        inner_parser.parse_ref(cmd, arg, &value.to_ascii_lowercase())
    }
}

impl ValueParserFactory for PackageId {
    type Parser = PackageIdParser;

    fn value_parser() -> Self::Parser {
        PackageIdParser
    }
}

#[derive(Clone, Debug)]
pub struct OptionalPackageIdParser;

impl TypedValueParser for OptionalPackageIdParser {
    type Value = OptionalPackageId;

    fn parse_ref(&self, cmd: &Command, arg: Option<&Arg>, value: &OsStr) -> Result<Self::Value, clap::error::Error> {
        let inner_parser = <Self::Value as std::str::FromStr>::from_str;
        inner_parser.parse_ref(cmd, arg, &value.to_ascii_lowercase())
    }
}

impl ValueParserFactory for OptionalPackageId {
    type Parser = OptionalPackageIdParser;

    fn value_parser() -> Self::Parser {
        OptionalPackageIdParser
    }
}
