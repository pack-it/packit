// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;

use crate::{
    cli::display::logging::warning,
    installer::types::PackageName,
    repositories::types::{Date, PackageMeta, PackageVersionMeta},
};

/// Shows a warning if the given PackageMeta is deprecated or will be deprecated soon.
pub fn show_package_warnings(package: &PackageMeta) {
    if let Some(deprecation) = &package.deprecation {
        let reason = match &deprecation.reason {
            Some(reason) => format!(" with reason '{reason}'"),
            None => String::default(),
        };

        let deprecated_from = &deprecation.deprecated_from;
        match *deprecated_from <= Date::now() {
            true => warning!("Package {} is deprecated since {}{reason}", package.name, deprecated_from),
            false => warning!("Package {} will be deprecated at {}{reason}", package.name, deprecated_from),
        }
    }
}

/// Shows a warning if the given PackageVersionMeta is deprecated or will be deprecated soon.
pub fn show_package_version_warnings(package_version: &PackageVersionMeta, package_name: &PackageName) {
    if let Some(deprecation) = &package_version.deprecation {
        let reason = match &deprecation.reason {
            Some(reason) => format!(" with reason '{reason}'"),
            None => String::default(),
        };

        let package_str = format!("{package_name}@{}", package_version.version).bold().blue(); // TODO: Maybe make a display util for this
        let deprecated_from = &deprecation.deprecated_from;
        match *deprecated_from <= Date::now() {
            true => warning!("Package version {package_str} is deprecated since {}{reason}", deprecated_from),
            false => warning!("Package version {package_str} will be deprecated at {}{reason}", deprecated_from),
        }
    }
}
