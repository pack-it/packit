use std::fs;

use thiserror::Error;

use crate::{config::Config, installer::types::PackageId, issue::Issue, storage::package_register::PackageRegister};

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Cannot read install directory")]
    IOError(#[from] std::io::Error),
}

pub struct Verifier<'a> {
    config: &'a Config,
    register: &'a PackageRegister,
}

impl<'a> Verifier<'a> {
    pub fn new(config: &'a Config, register: &'a PackageRegister) -> Self {
        Self { config, register }
    }

    pub fn find_issues(&self) -> Result<Vec<Issue>, VerifierError> {
        let mut issues = Vec::new();

        //self.check_packit_files();

        if let Some(issue) = self.check_consistency()? {
            issues.push(issue);
        }

        if let Some(issue) = self.check_dependency_tree() {
            issues.push(issue);
        }

        Ok(issues)
    }

    pub fn find_package_issue(&self) {
        // Find issues with a specific package, maybe even package it and compare it with checksum
        // (Not sure if checksum stays the same for a package or that data can be changed/added)
    }

    fn check_packit_files(&self) {
        // TODO: Check if all packit files (config, installed.toml, etc) are present and in the correct place
        // Can't check config because error is returned before find_issues is ran
    }

    fn check_consistency(&self) -> Result<Option<Issue>, VerifierError> {
        let mut missing = Vec::new();
        for package in self.register.iterate_all() {
            let package_directory = self
                .config
                .prefix_directory
                .join("packages")
                .join(&package.package_id.name)
                .join(package.package_id.version.to_string());

            // Check if the directory exists, if not add it to the missing packages
            if !fs::exists(package_directory)? {
                missing.push(package.package_id.clone());
            }
        }

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(Issue::InconsistentStorage(missing)))
    }

    fn check_dependency_tree(&self) -> Option<Issue> {
        let mut all_missing = Vec::new();
        for package in self.register.iterate_all() {
            all_missing.extend(self.check_package_dependency_tree(&package.package_id));
        }

        if all_missing.is_empty() {
            return None;
        }

        Some(Issue::BrokenTree(all_missing))
    }

    fn check_package_dependency_tree(&self, package_id: &PackageId) -> Vec<(PackageId, PackageId)> {
        let package = match self.register.get_package_version(package_id) {
            Some(package) => package,
            None => todo!("Some error"),
        };

        let mut missing = Vec::new();
        for dependency in &package.dependencies {
            if self.register.get_package_version(dependency).is_none() {
                missing.push((package_id.clone(), dependency.clone()));
                continue;
            }

            // Recursively check if all the dependencies are installed
            missing.extend(self.check_package_dependency_tree(dependency));
        }

        missing
    }
}
