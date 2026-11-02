// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    cli::display::logging::warning,
    platforms::{
        Target,
        tool_detection::{self, error::ToolDetectionError},
    },
    register::installed_package_version::InstalledPackageVersion,
    repositories::types::Requirement,
    utils::env::Environment,
};

#[cfg(target_family = "unix")]
const PATH_SEPARATOR: &str = ":";

#[cfg(target_os = "windows")]
const PATH_SEPARATOR: &str = ";";

/// The errors that occur during the creation of the script environment.
#[derive(Error, Debug)]
pub enum ScriptEnvError {
    #[error("Cannot add {} to script env {variable}: cannot convert PathBuf to string", path.display())]
    PathBufConversion {
        path: PathBuf,
        variable: String,
    },

    #[error("Requirement {0} is not satisfied")]
    RequirementNotSatisfied(Requirement),

    #[error("Cannot find tool {0} on the system")]
    ToolNotFound(String),

    #[error("Error while detecting tool on the system")]
    ToolDetectionError(#[from] ToolDetectionError),
}

pub type Result<T> = core::result::Result<T, ScriptEnvError>;

/// Holds all the data necessary to build a normalized script environment.
pub struct ScriptEnv<'a> {
    extra_paths: Vec<String>,
    dependencies: &'a Vec<&'a InstalledPackageVersion>,
    requirements: &'a Vec<Requirement>,
}

impl<'a> TryInto<Environment> for ScriptEnv<'a> {
    type Error = ScriptEnvError;

    /// Converts the `ScriptEnv` struct into a normalized `Environment` struct.
    fn try_into(self) -> Result<Environment> {
        let mut env = Environment::new();

        env.insert_var("PATH", self.create_path()?);

        // Add requirement specific vars to the script env
        env.expand(self.create_requirement_environment()?);

        Ok(env)
    }
}

impl<'a> ScriptEnv<'a> {
    /// Creates a new `ScriptEnv`.
    pub fn new(dependencies: &'a Vec<&'a InstalledPackageVersion>, requirements: &'a Vec<Requirement>) -> Self {
        Self::new_with_extra_paths(Vec::new(), dependencies, requirements)
    }

    /// Creates a new `ScriptEnv` with extra PATH entries.
    pub fn new_with_extra_paths(
        extra_paths: Vec<String>,
        dependencies: &'a Vec<&'a InstalledPackageVersion>,
        requirements: &'a Vec<Requirement>,
    ) -> Self {
        Self {
            extra_paths,
            dependencies,
            requirements,
        }
    }

    /// Creates the `PATH` for the `Environment`.
    /// The path will include the bin directories of all dependencies and standard system bin paths.
    fn create_path(&self) -> Result<String> {
        let mut parts = Vec::new();

        // Add all dependencies to PATH
        for dependency in self.dependencies {
            let bin_path = dependency.install_path.join("bin");

            // Skip adding if the bin dir does not exist
            if !bin_path.exists() {
                continue;
            }

            parts.push(Self::path_to_string(&bin_path, "PATH")?);
        }

        // Add all extra paths to the path
        for extra_path in &self.extra_paths {
            parts.push(extra_path.clone());
        }

        // Add standard Unix system bin paths to PATH
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            parts.append(&mut vec!["/usr/bin", "/bin", "/usr/sbin", "/sbin"].into_iter().map(String::from).collect());
        }

        // Add standard Windows system bin paths to PATH
        #[cfg(target_os = "windows")]
        {
            parts.append(
                &mut vec![
                    "C:\\Windows",
                    "C:\\Windows\\system32",
                    "C:\\Windows\\System32\\Wbem",
                    "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            );
        }

        Ok(parts.join(PATH_SEPARATOR))
    }

    /// Checks if the requirements are all satisfied.
    /// Returns a `ScriptEnvError::RequirementNotSatisfied` is not all requirements are satisfied.
    pub fn check_requirements_satisfied(&self) -> Result<()> {
        for requirement in self.requirements {
            if !requirement.is_satisfied()? {
                return Err(ScriptEnvError::RequirementNotSatisfied(requirement.clone()));
            }
        }

        Ok(())
    }

    /// Creates an environment from the given requirements.
    fn create_requirement_environment(&self) -> Result<Environment> {
        let mut environment = Environment::new();

        for requirement in self.requirements {
            match requirement {
                Requirement::Msvc { .. } => {
                    // Detect MSVC toolchain
                    let Some(msvc) = tool_detection::detect_msvc()? else {
                        return Err(ScriptEnvError::ToolNotFound("msvc".into()));
                    };

                    // Get arch, skip requirement if arch is `None`.
                    let Some(arch) = msvc.get_vcvarsall_arch(&Target::current()) else {
                        warning!("Tried to load MSVC for an unsupported target, skipping adding to env");
                        continue;
                    };

                    let vcvarsall_path = &msvc.get_vcvarsall_path();

                    // Add requirement specific environment vars to result
                    environment.insert_vars(HashMap::from([
                        ("PACKIT_VS_PATH", Self::path_to_string(msvc.get_vs_path(), "PACKIT_VS_PATH")?),
                        ("PACKIT_VCVARSALL", Self::path_to_string(vcvarsall_path, "PACKIT_VCVARSALL")?),
                        ("PACKIT_VCVARSALL_ARCH", arch.into()),
                        ("PACKIT_MSVC_VERSION", msvc.get_version().to_string()),
                    ]));
                },
            }
        }

        Ok(environment)
    }

    /// Converts a `PathBuf` to a string.
    /// Returns a `ScriptEnvError::PathBufConversion` if the path cannot be converted.
    pub fn path_to_string(path: &Path, env_var: &str) -> Result<String> {
        match path.to_str() {
            Some(string) => Ok(string.into()),
            None => Err(ScriptEnvError::PathBufConversion {
                path: path.to_path_buf(),
                variable: env_var.into(),
            }),
        }
    }
}
