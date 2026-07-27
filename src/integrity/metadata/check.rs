// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    cli::display::{ProgressBar, aligned_print::PairAligner, logging::error, styled::Styled},
    installer::types::{PackageId, PackageName},
    integrity::metadata::issue::{IssueType, MetaIssue},
    repositories::{
        error::RepositoryError,
        provider::MetadataProvider,
        types::{
            Checksum, IndexMeta, Licenses, PackageMeta, PackageTarget, PackageVersionMeta, Patch, RepositoryMeta, Source, Sources,
            TargetBounds,
        },
    },
    utils::{fuzzy, requests},
};

/// The `MetaCheck` is used during metadata checks and holds information about the issues.
pub struct MetaCheck {
    repository: String,
    provider: Box<dyn MetadataProvider>,
    issues: Vec<MetaIssue>,
    checks_skipped: bool,
}

impl MetaCheck {
    /// Constructs a new `MetaCheck` with the given repository and metadata provider.
    pub fn new(repository: &str, provider: Box<dyn MetadataProvider>) -> Self {
        Self {
            repository: repository.to_string(),
            provider,
            issues: Vec::new(),
            checks_skipped: false,
        }
    }

    /// Checks the metadata of the repository defined in `MetaCheck`. If a package name is given, only that package is checked.
    /// Otherwise the `index.toml` is used to check all the packages.
    pub fn check(&mut self, packages: &Vec<PackageName>) {
        let repository_meta = match self.provider.read_repository_metadata() {
            Ok(repository_meta) => repository_meta,
            Err(e) => {
                let description = format!("Repository metadata from '{}' could not be parsed", self.repository);
                let issue = MetaIssue::default(description);
                self.issues.push(issue.set_error(Box::new(e)).set_checks_skipped(true).set_issue_type(IssueType::Fatal));
                return;
            },
        };

        let index = match self.provider.read_index_metadata() {
            Ok(index_meta) => index_meta,
            Err(e) => {
                let description = format!("Repository 'index.toml' from '{}' cannot be parsed", self.repository);
                let issue = MetaIssue::default(description);
                self.issues.push(issue.set_error(Box::new(e)).set_checks_skipped(true).set_issue_type(IssueType::Fatal));
                return;
            },
        };

        let packages = match &packages.is_empty() {
            false => packages,
            true => &index.supported_packages.iter().cloned().collect(),
        };

        let mut progress_bar = ProgressBar::new(packages.len() as u64, "Checking".to_string());
        for (i, package_name) in packages.iter().enumerate() {
            let message = format!("Checking {}", package_name.style());
            progress_bar.adjust_prefix(message);
            progress_bar.set_position(i as u64);
            self.check_package_meta(&index, package_name, &repository_meta);
        }
    }

    /// Checks the metadata of a package. Then iterates over all specified versions to check those as well.
    fn check_package_meta(&mut self, index: &IndexMeta, package_name: &PackageName, repository_meta: &RepositoryMeta) {
        let Some(package_meta) = self.read_package_meta(index, package_name) else {
            return;
        };

        // Check if the package required Packit version is lower than the repository required Packit version
        if let Some(required_version) = &package_meta.required_packit_version
            && repository_meta.required_packit_version >= *required_version
        {
            self.issues.push(MetaIssue::default(format!(
                "The required Packit version for {} is lower than or equal to repository '{}' required Packit version",
                package_name.style(),
                self.repository
            )));
        }

        if let Some(homepage) = &package_meta.homepage {
            // Check if the homepage exists
            match requests::check_url(homepage) {
                Ok(exists) if !exists => {
                    let description = format!("The homepage URL '{homepage}' of {} does not exist", package_name.style());
                    self.issues.push(MetaIssue::default(description));
                },
                Ok(_) => {},
                Err(e) => error!(e, "Could not perform homepage URL existence check on {}", package_name.style()),
            }

            // Check if URL is https
            if !homepage.starts_with("https") {
                let description = format!("The homepage URL '{homepage}' of {} is not https", package_name.style());
                self.issues.push(MetaIssue::default(description).set_issue_type(IssueType::Warning));
            }
        }

        // Check that at least one version is specified
        if package_meta.versions.is_empty() {
            let description = format!("Package {} has no versions listed in its metadata", package_meta.name.style());
            self.issues.push(MetaIssue::default(description));
        }

        // Check that at least one target bound is specified
        if package_meta.supported_versions.keys().len() == 0 {
            let description = format!("Package {} has no target listed in its metadata", package_meta.name.style());
            self.issues.push(MetaIssue::default(description));
        }

        // Check that the version intervals for each target are non-empty
        for (target, version_interval) in &package_meta.supported_versions {
            if version_interval.is_empty() {
                self.issues.push(MetaIssue::default(format!(
                    "No version interval specified for target '{target}' from package {}",
                    package_meta.name.style()
                )));
            }
        }

        // Check conflict fields
        for conflict_package in &package_meta.conflicts_with {
            // Skip package if it cannot be found (a conflicting package can be in a different repository)
            let Ok(conflict_meta) = self.provider.read_package(conflict_package) else {
                continue;
            };

            if !conflict_meta.conflicts_with.contains(package_name) {
                self.issues.push(MetaIssue::default(format!(
                    "Conflict from package {} is not specified as conflict in package {}",
                    package_name.style(),
                    conflict_meta.name.style()
                )));
            }
        }

        // Check for disabled before deprecation in package meta
        if let Some(deprecation) = &package_meta.deprecation
            && let Some(disabled_from) = &deprecation.disabled_from
            && deprecation.deprecated_from > *disabled_from
        {
            self.issues.push(MetaIssue::default(format!(
                "Package {} is disabled on '{disabled_from}' before deprecation on '{}'",
                package_name.style(),
                deprecation.deprecated_from
            )));
        }

        // Check if listed versions exist and do package version specific metadata checks
        for version in &package_meta.versions {
            let package_id = PackageId::new(package_name.clone(), version.clone());
            let package_version = match self.provider.read_package_version(package_name, version) {
                Ok(package_version) => package_version,
                Err(e) => {
                    let description = format!("Package {} could not be parsed", package_id.style());
                    let issue = MetaIssue::default(description);
                    self.issues.push(issue.set_error(Box::new(e)).set_checks_skipped(true));
                    continue;
                },
            };

            // Check if the package version required Packit version is lower than the repository required Packit version
            if let Some(required_version) = &package_version.required_packit_version
                && repository_meta.required_packit_version >= *required_version
            {
                self.issues.push(MetaIssue::default(format!(
                    "The required Packit version for {} is lower than or equal to the required version in repository '{}'",
                    package_name.style(),
                    self.repository
                )));
            }

            // Check if the package version required Packit version is lower than the package required Packit version
            if let Some(package_required_version) = &package_meta.required_packit_version
                && let Some(required_version) = &package_version.required_packit_version
                && package_required_version >= required_version
            {
                self.issues.push(MetaIssue::default(format!(
                    "The required Packit version for package {} is higher than or equal to the required version in package version {}",
                    package_name.style(),
                    package_id.style()
                )));
            }

            // Check if the version exists in any of the supported ranges
            if !package_meta.supported_versions.values().any(|i| i.covers(version)) {
                self.issues.push(MetaIssue::default(format!(
                    "Version {} in {} doesn't exist in any target support range",
                    version.style(),
                    package_name.style()
                )));
            }

            self.check_deprecation_dates(&package_id, &package_meta, &package_version);

            self.check_package_version_meta(package_name, &package_version);
        }
    }

    /// Reads the package metadata. Returns the package metadata if it can be found and read. If the package
    /// could not be found a package not found issue is created. If the package could not be parsed a parse
    /// issue is created. For any other errors the error is immediately printed.
    fn read_package_meta(&mut self, index: &IndexMeta, package_name: &PackageName) -> Option<PackageMeta> {
        match self.provider.read_package(package_name) {
            Ok(package) => return Some(package),
            Err(RepositoryError::IOError(..)) | Err(RepositoryError::UnsuccessfulRequest(..)) => {
                let fuzzy_match = fuzzy::index_search(index, package_name);

                #[expect(clippy::manual_map)]
                let suggestion = match fuzzy_match {
                    Some(fuzzy_match) => Some(fuzzy_match.style().to_string()),
                    None => None,
                };

                let description = format!("Package {} could not be found", package_name.style());
                let issue = MetaIssue::default(description).set_checks_skipped(true).set_suggestion(suggestion);
                self.issues.push(issue);
            },
            Err(RepositoryError::ParseError(e)) => {
                let description = format!("Package {} could not be parsed", package_name.style());
                let issue = MetaIssue::default(description);
                self.issues.push(issue.set_error(Box::new(e)));
            },
            Err(e) => error!(e, "Cannot read package {}", package_name.style()),
        };

        None
    }

    /// Checks the metadata of a package version. Then iterates over all specified targets to check those as well.
    fn check_package_version_meta(&mut self, package_name: &PackageName, package_version_meta: &PackageVersionMeta) {
        let package_id = PackageId::new(package_name.clone(), package_version_meta.version.clone());

        // Check license
        self.check_license(&package_version_meta.license, &package_id);

        // Check sources
        let sources = match &package_version_meta.sources {
            Sources::Single(source) => vec![("all", source)],
            Sources::Named(sources) => sources.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        };

        // Check if the sources aren't empty
        if sources.is_empty() {
            let description = format!("No sources for package {}", package_id.style());
            self.issues.push(MetaIssue::default(description));
        }

        // Check all sources
        for (target, source) in sources {
            self.check_source(&package_id, target, source);
        }

        // Check if the targets aren't empty
        if package_version_meta.targets.is_empty() {
            let description = format!("No targets for package {}", package_id.style());
            self.issues.push(MetaIssue::default(description));
        }

        // Check if external test files exist
        for file in &package_version_meta.external_test_files {
            if !matches!(self.provider.read_file_bytes(package_name, file), Ok(Some(_))) {
                let description = format!("External file '{file}' specified in {} could not be found", package_id.style());
                self.issues.push(MetaIssue::default(description));
            }
        }

        // Check all targets
        for (bounds, target) in &package_version_meta.targets {
            self.check_target(bounds, target, &package_version_meta.sources, &package_id);

            // Check if there are duplicates between the package version and target fields
            for dependency in &target.dependencies {
                if package_version_meta.dependencies.iter().any(|d| d.get_name() == dependency.get_name()) {
                    let description = format!("Duplicate dependency '{dependency}' found in {}", package_id.style());
                    self.issues.push(MetaIssue::default(description));
                }
            }

            for dependency in &target.build_dependencies {
                if package_version_meta.build_dependencies.iter().any(|d| d.get_name() == dependency.get_name()) {
                    let description = format!("Duplicate build dependency '{dependency}' found in {}", package_id.style());
                    self.issues.push(MetaIssue::default(description));
                }
            }

            if let Some(skip_symlinking) = target.skip_symlinking {
                if package_version_meta.skip_symlinking || !skip_symlinking {
                    self.issues.push(MetaIssue::default(format!(
                        "Field 'skip_symlinking' unnecessarily specified on target '{bounds}' in {}",
                        package_id.style()
                    )));
                }
            }

            for file in &target.external_test_files {
                if package_version_meta.external_test_files.contains(file) {
                    self.issues.push(MetaIssue::default(format!(
                        "Duplicate external test file '{file}' found in target '{bounds}' in {}",
                        package_id.style()
                    )));
                }
            }

            for (key, value) in &target.script_args {
                let Some(other_value) = package_version_meta.script_args.get(key) else {
                    continue;
                };

                if other_value == value {
                    self.issues.push(MetaIssue::default(format!(
                        "Duplicate script arg '{key} = {value}' found in target '{bounds}' in {}",
                        package_id.style()
                    )));
                }
            }
        }
    }

    /// Checks a specific source specified in a package version. Then iterates over all specified patches to check those as well.
    fn check_source(&mut self, package_id: &PackageId, target: &str, source: &Source) {
        for url in source.mirrors.iter().chain(std::iter::once(&source.url)) {
            // Check source URL existence
            let response = match requests::get(url) {
                Ok(response) if response.status().is_success() => response,
                _ => {
                    let description = format!("The URL '{url}' of {} target '{target}' does not exist", package_id.style());
                    self.issues.push(MetaIssue::default(description).set_checks_skipped(true));
                    continue;
                },
            };

            // Check if URL is https
            if !url.starts_with("https") {
                let description = format!("The URL '{url}' of {} target '{target}' is not https", package_id.style());
                self.issues.push(MetaIssue::default(description).set_issue_type(IssueType::Warning));
            }

            // Get bytes from response
            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(e, "Unable to get file bytes");
                    self.checks_skipped = true;
                    continue;
                },
            };

            // Check source checksum
            let correct_checksum = Checksum::from_bytes(&bytes);
            if source.checksum != correct_checksum {
                let description = format!(
                    "Checksum '{}' of {} in target '{target}' with url '{url}' is incorrect",
                    source.checksum,
                    package_id.style(),
                );

                self.issues.push(MetaIssue::default(description).set_suggestion(Some(correct_checksum.to_string())));
            }

            // Check source bytes
            if source.size.0 != bytes.len() as u32 {
                self.issues.push(MetaIssue::default(format!(
                    "Size '{}' of {} in target '{target}' with url '{url}' is incorrect",
                    source.size,
                    package_id.style(),
                )));
            }
        }

        // Check all source patches
        for (patch_number, patch) in &source.patches {
            self.check_patch(package_id, patch_number, patch, target);
        }
    }

    /// Checks a specific patch specified in a source.
    fn check_patch(&mut self, package_id: &PackageId, patch_number: &u32, patch: &Patch, target: &str) {
        // Check all patch URL's
        for url in patch.mirrors.iter().chain(std::iter::once(&patch.url)) {
            // Check source URL existence
            let response = match requests::get(url) {
                Ok(response) if response.status().is_success() => response,
                _ => {
                    let description = format!(
                        "The URL '{url}' of {} in target '{target}' patch {patch_number} does not exist",
                        package_id.style(),
                    );

                    self.issues.push(MetaIssue::default(description).set_checks_skipped(true));
                    continue;
                },
            };

            // Check if URL is https
            if !url.starts_with("https") {
                let description = format!(
                    "The URL '{url}' of {} in target '{target}' patch {patch_number} is not https",
                    package_id.style(),
                );
                self.issues.push(MetaIssue::default(description).set_issue_type(IssueType::Warning));
            }

            // Get bytes from response
            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(e, "Unable to get file bytes");
                    self.checks_skipped = true;
                    continue;
                },
            };

            // Check source checksum
            let correct_checksum = Checksum::from_bytes(&bytes);
            if patch.checksum != correct_checksum {
                let description = format!(
                    "Checksum '{}' of {} in target '{target}' patch {patch_number} with url '{url}' is incorrect",
                    patch.checksum,
                    package_id.style(),
                );

                self.issues.push(MetaIssue::default(description).set_suggestion(Some(correct_checksum.to_string())));
            }
        }
    }

    /// Checks a specific target.
    fn check_target(&mut self, bounds: &TargetBounds, target: &PackageTarget, sources: &Sources, package_id: &PackageId) {
        // Check if externel test files exist
        for file in &target.external_test_files {
            if !matches!(self.provider.read_file_bytes(&package_id.name, file), Ok(Some(_))) {
                self.issues.push(MetaIssue::default(format!(
                    "External file '{file}' specified in {} for target '{bounds}' could not be found",
                    package_id.style(),
                )));
            }
        }

        // Check if there is a reference in the target when sources are target specific
        let source_reference = match &target.source {
            Some(source_reference) => source_reference,
            None if matches!(sources, Sources::Single(..)) => return,
            None => {
                self.issues.push(MetaIssue::default(format!(
                    "No source reference found in {} for target '{bounds}', eventhough sources are target specific",
                    package_id.style()
                )));

                return;
            },
        };

        // Check if the source references in the target are required and can be found
        match &sources {
            Sources::Single(_) => self.issues.push(MetaIssue::default(format!(
                "Found source reference '{source_reference}' in {} for target '{bounds}', eventhough none was required",
                package_id.style()
            ))),
            Sources::Named(sources) if !sources.contains_key(source_reference) => self.issues.push(MetaIssue::default(format!(
                "Source reference '{source_reference}' for target '{bounds}' could not be found in package version metadata of {}",
                package_id.style()
            ))),
            Sources::Named(_) => {},
        }
    }

    /// Checks the licenses of a package.
    fn check_license(&mut self, license: &Licenses, package_id: &PackageId) {
        let (license, exceptions) = match &license {
            Licenses::Unknown => return,
            Licenses::Single(license) => (license, None),
            Licenses::SingleWithExceptions { name, exceptions } => (name, Some(exceptions)),
            Licenses::Any { any: licenses } | Licenses::All { all: licenses } => {
                // Check if the list of licenses is empty
                if licenses.is_empty() {
                    let description = format!("List of licenses from {} is empty", package_id.style());
                    self.issues.push(MetaIssue::default(description));
                }

                // Check if list contains only a single license
                if licenses.len() == 1 {
                    let description = format!("List of licenses from {} contains only a single license", package_id.style());
                    self.issues.push(MetaIssue::default(description));
                }

                // Check each license in the list
                for license in licenses {
                    self.check_license(license, package_id);
                }
                return;
            },
        };

        // Check if the license is an empty string
        if license.is_empty() {
            let description = format!("Package {} has an empty license", package_id.style());
            self.issues.push(MetaIssue::default(description));
        }

        // Check exceptions if specified
        if let Some(exceptions) = exceptions {
            // Check if the list of exceptions is empty
            if exceptions.is_empty() {
                let description = format!("List of license exceptions from {} is empty", package_id.style());
                self.issues.push(MetaIssue::default(description));
            }

            // Check if one of the excptions is an empty string
            for exception in exceptions {
                if exception.is_empty() {
                    let description = format!("Package {} has an empty license exception", package_id.style());
                    self.issues.push(MetaIssue::default(description));
                }
            }
        }
    }

    /// Checks the deprecation (and disable) dates of a package and a package version.
    /// The dates from the package meta and package version meta are compared with each other.
    fn check_deprecation_dates(&mut self, package_id: &PackageId, package_meta: &PackageMeta, package_version_meta: &PackageVersionMeta) {
        // Check for disabled before deprecation in package version meta
        if let Some(deprecation) = &package_version_meta.deprecation
            && let Some(disabled_from) = &deprecation.disabled_from
            && deprecation.deprecated_from > *disabled_from
        {
            self.issues.push(MetaIssue::default(format!(
                "Package {} is disabled on '{disabled_from}' before deprecation on '{}'",
                package_id.style(),
                deprecation.deprecated_from
            )));
        }

        // Check deprecation dates
        let Some(package_deprecation) = &package_meta.deprecation else {
            return;
        };

        let Some(version_deprecation) = &package_version_meta.deprecation else {
            return;
        };

        // Check if the package deprecates before the package version
        if package_deprecation.deprecated_from <= version_deprecation.deprecated_from {
            self.issues.push(MetaIssue::default(format!(
                "The deprecation at '{}' of package {} happens earlier than package version {} at '{}'",
                package_deprecation.deprecated_from,
                package_id.name.style(),
                package_id.style(),
                version_deprecation.deprecated_from
            )));
        }

        // Check disabled dates
        let Some(package_disabled_from) = &package_deprecation.disabled_from else {
            return;
        };

        let Some(version_disabled_from) = &version_deprecation.disabled_from else {
            return;
        };

        // Check if the package disables before the package version
        if package_disabled_from <= version_disabled_from {
            self.issues.push(MetaIssue::default(format!(
                "The package {} disabled from '{package_disabled_from}' is earlier than package version {} disabled from '{version_disabled_from}'",
                package_id.name.style(),
                package_id.style(),
            )));
        }
    }

    /// Displays the issues which have been found.
    pub fn display_issues(&self) {
        if self.issues.is_empty() {
            println!("No issues were found!");
            return;
        }

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

        if self.checks_skipped {
            println!("Some checks were skipped due to errors NOT created by invalid metadata");
        }
    }
}
