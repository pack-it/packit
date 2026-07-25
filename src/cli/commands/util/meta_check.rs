// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::HashSet, error::Error, fmt::Display, fs, process::exit};

use clap::Args;
use colored::Colorize;
use url::Url;

use crate::{
    cli::{
        commands::HandleCommand,
        display::{aligned_print::PairAligner, logging::error, not_found, styled::Styled},
    },
    config::{Config, Repository},
    installer::types::{PackageId, PackageName},
    repositories::{
        error::RepositoryError,
        provider::{self, MetadataProvider},
        types::{
            Checksum, IndexMeta, Licenses, PackageMeta, PackageTarget, PackageVersionMeta, Patch, RepositoryMeta, Source, Sources,
            TargetBounds,
        },
    },
    utils::{requests, unwrap_or_exit::UnwrapOrExit},
};

/// Checks the metedata of the given package in a repository or all packages in a repository if no package has been given.
#[derive(Args, Debug)]
pub struct MetaCheckArgs {
    /// The repository of the package(s). Can be a repository id specified in `Config.toml`, a path to a repo or a URL to a repo
    repository: String,

    /// The package metadata to check
    package_name: Option<PackageName>,
}

impl HandleCommand for MetaCheckArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit(1);
        let repository = self.get_repository(&config);
        let provider = provider::create_metadata_provider(&repository).unwrap_or_exit_msg("Could not create metadata provider", 1);
        let mut meta_checker = MetaCheck::new(&self.repository, provider);
        meta_checker.check(self.package_name.clone());
        meta_checker.display_issues();
    }
}

impl MetaCheckArgs {
    fn get_repository(&self, config: &Config) -> Repository {
        if let Some(repository) = config.repositories.get(&self.repository) {
            return repository.clone();
        }

        // Return web repository if the string is a valid URL
        if Url::parse(&self.repository).is_ok() {
            return Repository::new(&self.repository, "web");
        }

        // Return filesystem repository if the string exists as a path
        if matches!(fs::exists(&self.repository), Ok(true)) {
            return Repository::new(&self.repository, "fs");
        }

        error!(msg: "Wrong repository '{}', please use a valid repository id, URL or path to a repository", self.repository);
        exit(1)
    }
}

struct MetaIssue {
    issue_type: IssueType,
    description: String,
    checks_skipped: bool,
    error: Option<Box<dyn Error>>,
}

impl MetaIssue {
    pub fn default(description: &str) -> Self {
        Self {
            issue_type: IssueType::Breaking,
            description: description.to_string(),
            checks_skipped: false,
            error: None,
        }
    }

    pub fn set_issue_type(mut self, issue_type: IssueType) -> Self {
        self.issue_type = issue_type;
        self
    }

    pub fn set_checks_skipped(mut self, checks_skipped: bool) -> Self {
        self.checks_skipped = checks_skipped;
        self
    }

    pub fn set_error(mut self, error: Box<dyn Error>) -> Self {
        self.error = Some(error);
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

enum IssueType {
    // Cannot continue with checks
    Fatal,

    // The metadata contains information which breaks certain logic, but continueing is possible
    Breaking,

    // The metadata is correct and functions, however it's unconventional (although maybe unavoidable)
    Warning,
}

struct MetaCheck {
    repository: String,
    provider: Box<dyn MetadataProvider>,
    issues: Vec<MetaIssue>,
}

impl MetaCheck {
    pub fn new(repository: &str, provider: Box<dyn MetadataProvider>) -> Self {
        Self {
            repository: repository.to_string(),
            provider,
            issues: Vec::new(),
        }
    }

    fn check(&mut self, package_name: Option<PackageName>) {
        let repository_meta = match self.provider.read_repository_metadata() {
            Ok(repository_meta) => repository_meta,
            Err(e) => {
                let description = format!("Repository metadata from '{}' could not be parsed", self.repository);
                let issue = MetaIssue::default(&description);
                self.issues.push(issue.set_error(Box::new(e)).set_checks_skipped(true).set_issue_type(IssueType::Fatal));
                return;
            },
        };

        let index = match self.provider.read_index_metadata() {
            Ok(index_meta) => index_meta,
            Err(e) => {
                let description = format!("Repository 'index.toml' from '{}' cannot be parsed", self.repository);
                let issue = MetaIssue::default(&description);
                self.issues.push(issue.set_error(Box::new(e)).set_checks_skipped(true).set_issue_type(IssueType::Fatal));
                return;
            },
        };

        let packages = match &package_name {
            Some(package) => &HashSet::from([package.clone()]),
            None => &index.supported_packages,
        };

        for package_name in packages {
            self.check_package_meta(&index, &package_name, &repository_meta);
        }
    }

    fn check_package_meta(&mut self, index: &IndexMeta, package_name: &PackageName, repository_meta: &RepositoryMeta) {
        let Some(package_meta) = self.read_package_meta(index, package_name) else {
            return;
        };

        // Check if the package required Packit version is lower then the repository required Packit version
        if let Some(required_version) = &package_meta.required_packit_version
            && repository_meta.required_packit_version >= *required_version
        {
            let description = format!(
                "The required Packit version for {} is lower then or equal to repository '{}' required Packit version",
                package_name.style(),
                self.repository
            );
            self.issues.push(MetaIssue::default(&description));
        }

        if let Some(homepage) = &package_meta.homepage {
            if !requests::check_url(homepage).unwrap_or_exit(1) {
                let description = format!("The homepage URL of {} does not exist", package_name.style());
                self.issues.push(MetaIssue::default(&description));
            }

            // Check if URL is https
            if !homepage.starts_with("https") {
                let description = format!("The homepage URL '{}' of {} is not https", homepage, package_name.style());
                self.issues.push(MetaIssue::default(&description).set_issue_type(IssueType::Warning));
            }
        }

        // Check that at least one version is specified
        if package_meta.versions.is_empty() {
            let description = format!("Package {} has no versions listed in its metadata", package_meta.name.style());
            self.issues.push(MetaIssue::default(&description));
        }

        // Check that at least one target bound is specified
        if package_meta.supported_versions.keys().len() == 0 {
            let description = format!("Package {} has no target listed in its metadata", package_meta.name.style());
            self.issues.push(MetaIssue::default(&description));
        }

        // Check that the version intervals for each target are non-empty
        for (target, version_interval) in &package_meta.supported_versions {
            if version_interval.is_empty() {
                let description = format!(
                    "No version interval specified for target '{}' from package {}",
                    target,
                    package_meta.name.style()
                );
                self.issues.push(MetaIssue::default(&description));
            }
        }

        // Check conflict fields
        for conflict_package in &package_meta.conflicts_with {
            let Some(conflict_meta) = self.read_package_meta(index, conflict_package) else {
                continue;
            };

            if !conflict_meta.conflicts_with.contains(package_name) {
                let description = format!(
                    "Conflict from package {} is not specified as conflict in package {}",
                    package_name.style(),
                    conflict_meta.name.style()
                );
                self.issues.push(MetaIssue::default(&description));
            }
        }

        // Check if listed versions exist (cannot be parsed) and do package version specific metadata checks
        for version in &package_meta.versions {
            let package_id = PackageId::new(package_name.clone(), version.clone());
            let package_version = match self.provider.read_package_version(package_name, &version) {
                Ok(package_version) => package_version,
                Err(e) => {
                    let description = format!("Package {} could not be parsed", package_id.style());
                    let issue = MetaIssue::default(&description);
                    self.issues.push(issue.set_error(Box::new(e)).set_checks_skipped(true));
                    continue;
                },
            };

            // Check if the package version required Packit version is lower then the repository required Packit version
            if let Some(required_version) = &package_version.required_packit_version
                && repository_meta.required_packit_version >= *required_version
            {
                let description = format!(
                    "The required Packit version for {} is lower then or equal to the required version in repository '{}'",
                    package_name.style(),
                    self.repository
                );
                self.issues.push(MetaIssue::default(&description));
            }

            // Check if the package version required Packit version is lower then the package required Packit version
            if let Some(package_required_version) = &package_meta.required_packit_version
                && let Some(required_version) = &package_version.required_packit_version
                && package_required_version <= required_version
            {
                let description = format!(
                    "The required Packit version for package {} is lower then or equal to the required version in package version {}",
                    package_name.style(),
                    package_id.style()
                );
                self.issues.push(MetaIssue::default(&description));
            }

            // Check if the version exists in any of the supported ranges
            if !package_meta.supported_versions.values().any(|i| i.covers(version)) {
                let description = format!(
                    "Version {} in {} doesn't exist in any target support range",
                    version.style(),
                    package_name.style()
                );
                self.issues.push(MetaIssue::default(&description));
            }

            self.check_deprecation_dates(&package_id, &package_meta, &package_version);

            self.check_package_version_meta(&package_name, &package_version);
        }
    }

    fn read_package_meta(&mut self, index: &IndexMeta, package_name: &PackageName) -> Option<PackageMeta> {
        match self.provider.read_package(&package_name) {
            Ok(package) => return Some(package),
            Err(RepositoryError::IOError(..)) => not_found::index_package(package_name, index),
            Err(e) => {
                let description = format!("Package {} could not be parsed", package_name.style());
                let issue = MetaIssue::default(&description);
                self.issues.push(issue.set_error(Box::new(e)));
            },
        };

        return None;
    }

    fn check_package_version_meta(&mut self, package_name: &PackageName, package_version_meta: &PackageVersionMeta) {
        let package_id = PackageId::new(package_name.clone(), package_version_meta.version.clone());

        // Check license
        self.check_license(&package_version_meta.license, &package_id);

        // Check sources
        let sources = match &package_version_meta.sources {
            Sources::Single(source) => vec![("all", source)],
            Sources::Named(sources) => sources.into_iter().map(|(k, v)| (k.as_str(), v)).collect(),
        };

        // Check if the sources aren't empty
        if sources.is_empty() {
            let description = format!("No sources for package {}", package_id.style());
            self.issues.push(MetaIssue::default(&description));
        }

        // Check all sources
        for (target, source) in sources {
            self.check_source(&package_id, target, source);
        }

        // Check if the targets aren't empty
        if package_version_meta.targets.is_empty() {
            let description = format!("No targets for package {}", package_id.style());
            self.issues.push(MetaIssue::default(&description));
        }

        // Check if externel test files exist
        for file in &package_version_meta.external_test_files {
            if !matches!(self.provider.read_file_bytes(package_name, file), Ok(Some(_))) {
                let description = format!("External file '{}' specified in {} could not be found", file, package_id.style());
                self.issues.push(MetaIssue::default(&description));
            }
        }

        // Check all targets
        for (bounds, target) in &package_version_meta.targets {
            self.check_target(bounds, target, &package_version_meta.sources, &package_id);

            // Check if there are duplicates between the package version and target fields
            for dependency in &target.dependencies {
                if package_version_meta.dependencies.iter().any(|d| d.get_name() == dependency.get_name()) {
                    let description = format!("Duplicate dependency '{}' found in {}", dependency, package_id.style());
                    self.issues.push(MetaIssue::default(&description));
                }
            }

            for dependency in &target.build_dependencies {
                if package_version_meta.build_dependencies.iter().any(|d| d.get_name() == dependency.get_name()) {
                    let description = format!("Duplicate build dependency '{}' found in {}", dependency, package_id.style());
                    self.issues.push(MetaIssue::default(&description));
                }
            }

            if let Some(skip_symlinking) = target.skip_symlinking {
                if package_version_meta.skip_symlinking || !skip_symlinking {
                    let description = format!("Field 'skip_symlinking' unnecessarily specified on target '{}'", bounds);
                    self.issues.push(MetaIssue::default(&description));
                }
            }

            for file in &target.external_test_files {
                if package_version_meta.external_test_files.contains(file) {
                    let description = format!("Duplicate external test file '{}' found in {}", file, package_id.style());
                    self.issues.push(MetaIssue::default(&description));
                }
            }

            for (key, value) in &target.script_args {
                if let Some(other_value) = package_version_meta.script_args.get(key) {
                    if other_value == value {
                        let description = format!("Duplicate script arg '{} = {}' found in {}", key, value, package_id.style());
                        self.issues.push(MetaIssue::default(&description));
                    }
                }
            }
        }
    }

    fn check_source(&mut self, package_id: &PackageId, target: &str, source: &Source) {
        for url in source.mirrors.iter().chain(std::iter::once(&source.url)) {
            // Check source URL existence
            let response = match requests::get(url) {
                Ok(response) if response.status().is_success() => response,
                _ => {
                    let description = format!("The URL '{}' of {} target '{}' does not exist", url, package_id.style(), target);
                    self.issues.push(MetaIssue::default(&description).set_checks_skipped(true));
                    continue;
                },
            };

            // Check if URL is https
            if !url.starts_with("https") {
                let description = format!("The URL '{}' of {} target '{}' is not https", url, package_id.style(), target);
                self.issues.push(MetaIssue::default(&description).set_issue_type(IssueType::Warning));
            }

            // Get bytes from response
            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(e, "Unable to get file bytes");
                    continue;
                },
            };

            // Check source checksum
            if source.checksum != Checksum::from_bytes(&bytes) {
                let description = format!(
                    "Checksum '{}' of {} target '{}' with url '{}' is incorrect",
                    source.checksum,
                    package_id.style(),
                    target,
                    url
                );
                self.issues.push(MetaIssue::default(&description));
            }

            // Check source bytes
            if source.size.0 != bytes.len() as u32 {
                let description = format!(
                    "Size '{}' of {} target '{}' with url '{}' is incorrect",
                    source.size,
                    package_id.style(),
                    target,
                    url
                );
                self.issues.push(MetaIssue::default(&description));
            }
        }

        // Check all source patches
        for (patch_number, patch) in &source.patches {
            self.check_patch(package_id, patch_number, patch, target);
        }
    }

    fn check_patch(&mut self, package_id: &PackageId, patch_number: &u32, patch: &Patch, target: &str) {
        // Check all patch URL's
        for url in patch.mirrors.iter().chain(std::iter::once(&patch.url)) {
            // Check source URL existence
            let response = match requests::get(url) {
                Ok(response) if response.status().is_success() => response,
                _ => {
                    let description = format!(
                        "The URL '{}' of {} target '{}' patch {} does not exist",
                        url,
                        package_id.style(),
                        target,
                        patch_number
                    );

                    self.issues.push(MetaIssue::default(&description).set_checks_skipped(true));
                    continue;
                },
            };

            // Check if URL is https
            if !url.starts_with("https") {
                let description = format!(
                    "The URL '{}' of {} target '{}' patch {} is not https",
                    url,
                    package_id.style(),
                    target,
                    patch_number
                );
                self.issues.push(MetaIssue::default(&description).set_issue_type(IssueType::Warning));
            }

            // Get bytes from response
            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(e) => {
                    // TODO: Checks are still skipped (occurs somewhere else to), but not a Packit metadata issue
                    error!(e, "Unable to get file bytes");
                    continue;
                },
            };

            // Check source checksum
            if patch.checksum != Checksum::from_bytes(&bytes) {
                let description = format!(
                    "Checksum '{}' of {} target '{}' patch {} with url '{}' is incorrect",
                    patch.checksum,
                    package_id.style(),
                    target,
                    patch_number,
                    url
                );
                self.issues.push(MetaIssue::default(&description));
            }
        }
    }

    fn check_target(&mut self, bounds: &TargetBounds, target: &PackageTarget, sources: &Sources, package_id: &PackageId) {
        // Check if externel test files exist
        for file in &target.external_test_files {
            if !matches!(self.provider.read_file_bytes(&package_id.name, file), Ok(Some(_))) {
                let description = format!("External file '{}' specified for target '{}' could not be found", file, bounds);
                self.issues.push(MetaIssue::default(&description));
            }
        }

        // Check if the source reference in the given target is required, or not present when it should be
        match &target.source {
            Some(source_reference) => match &sources {
                Sources::Single(_) => {
                    let description =
                        format!("Found source reference '{source_reference}' for target '{bounds}', eventhough none was required");
                    self.issues.push(MetaIssue::default(&description));
                },
                Sources::Named(sources) if !sources.contains_key(source_reference) => {
                    let description = format!(
                        "Source reference '{source_reference}' for target '{bounds}' could not be found in package version metadata"
                    );
                    self.issues.push(MetaIssue::default(&description));
                },
                Sources::Named(_) => {},
            },
            None if matches!(sources, Sources::Single(..)) => {},
            None => {
                let description = format!("No source reference found in target, eventhough sources are target specific");
                self.issues.push(MetaIssue::default(&description));
            },
        }
    }

    fn check_license(&mut self, license: &Licenses, package_id: &PackageId) {
        let licenses = match &license {
            Licenses::Unknown => return,
            Licenses::Single(license) => &vec![license.clone()],
            Licenses::Any { any } => any,
            Licenses::All { all } => all,
        };

        if licenses.is_empty() {
            let description = format!("License from {} not specified as unknown, but is empty", package_id.style());
            self.issues.push(MetaIssue::default(&description));
        }

        for license in licenses {
            if license.is_empty() {
                let description = format!("Single license is empty in {}", package_id.style());
                self.issues.push(MetaIssue::default(&description));
            }
        }
    }

    fn check_deprecation_dates(&mut self, package_id: &PackageId, package_meta: &PackageMeta, package_version_meta: &PackageVersionMeta) {
        // Check for disabled before deprecation in package meta
        if let Some(deprecation) = &package_meta.deprecation
            && let Some(disabled_from) = &deprecation.disabled_from
            && deprecation.deprecated_from > *disabled_from
        {
            let description = format!(
                "Package {} is disabled on '{}' before deprecation on '{}'",
                package_id.name.style(),
                disabled_from,
                deprecation.deprecated_from
            );
            self.issues.push(MetaIssue::default(&description));
        }

        // Check for disabled before deprecation in package version meta
        if let Some(deprecation) = &package_version_meta.deprecation
            && let Some(disabled_from) = &deprecation.disabled_from
            && deprecation.deprecated_from > *disabled_from
        {
            let description = format!(
                "Package {} is disabled on '{}' before deprecation on '{}'",
                package_id.style(),
                disabled_from,
                deprecation.deprecated_from
            );
            self.issues.push(MetaIssue::default(&description));
        }

        // Check deprecation dates
        let Some(package_deprecation) = &package_meta.deprecation else {
            return;
        };

        let Some(version_deprecation) = &package_version_meta.deprecation else {
            return;
        };

        if package_deprecation.deprecated_from <= version_deprecation.deprecated_from {
            let description = format!(
                "The deprecation at '{}' of package {} happens earlier then package version {} at '{}'",
                package_deprecation.deprecated_from,
                package_id.name.style(),
                package_id.style(),
                version_deprecation.deprecated_from
            );
            self.issues.push(MetaIssue::default(&description));
        }

        // Check disabled dates
        let Some(disabled_from) = &package_deprecation.disabled_from else {
            return;
        };

        let Some(version_disabled_from) = &version_deprecation.disabled_from else {
            return;
        };

        if disabled_from <= version_disabled_from {
            let description = format!(
                "The package {} disabled from '{}' is earlier then package version {} disabled from '{}'",
                package_id.name.style(),
                disabled_from,
                package_id.style(),
                version_disabled_from
            );
            self.issues.push(MetaIssue::default(&description));
        }
    }

    pub fn display_issues(&self) {
        if self.issues.is_empty() {
            println!("No issues were found!");
            return;
        }

        // TODO: Issue overview
        let mut count_fatal = 0;
        let mut count_breaking = 0;
        let mut count_warning = 0;
        for issue in &self.issues {
            match issue.issue_type {
                IssueType::Fatal => count_fatal += 1,
                IssueType::Breaking => count_breaking += 1,
                IssueType::Warning => count_warning += 1,
            }
        }

        println!("The following metadata issue were found:");
        let mut pair_aligner = PairAligner::new();
        pair_aligner.add("Fatal", count_fatal);
        pair_aligner.add("Breaking", count_breaking);
        pair_aligner.add("Warnings", count_warning);
        pair_aligner.display(PairAligner::VERTICAL_LINE_PREFIX);
        println!();

        for issue in &self.issues {
            println!("{issue}");

            if matches!(issue.issue_type, IssueType::Fatal) {
                return;
            }
        }
    }
}
