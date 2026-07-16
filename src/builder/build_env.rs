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
    register::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    repositories::types::Requirement,
    utils::env::Environment,
};

// TODO: We should probably also strip tokens from the env
#[rustfmt::skip]
const STRIPPED_VARS: &[&str] = &[
    "CC", "CXX", "OBJC", "OBJCXX", "CPP", "MAKE", "LD", "LDSHARED", "AR", "AS", "NM", "STRIP", "RANLIB", 
    "OBJCOPY", "CDPATH", "CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH", "OBJC_INCLUDE_PATH",
    "CFLAGS", "CXXFLAGS", "OBJCFLAGS", "OBJCXXFLAGS", "LDFLAGS", "CPPFLAGS", "ASFLAGS", "MAKEFLAGS",
    "CMAKE_INCLUDE_PATH", "CMAKE_FRAMEWORK_PATH", "CMAKE_TOOLCHAIN_FILE", "LIBRARY_PATH",
    "LD_LIBRARY_PATH", "LD_PRELOAD", "LD_RUN_PATH", "DYLD_LIBRARY_PATH", "DYLD_INSERT_LIBRARIES",
    "DYLD_FRAMEWORK_PATH", "DYLD_FALLBACK_LIBRARY_PATH", "DYLD_FALLBACK_FRAMEWORK_PATH", "PKG_CONFIG_SYSROOT_DIR",
    "PYTHONPATH", "PYTHONHOME", "PERL5LIB", "PERL_MB_OPT", "PERL_MM_OPT", "RUBYLIB", "NODE_PATH", 
    "CARGO_HOME", "RUSTUP_HOME", "RUSTFLAGS", "GOBIN", "GOPATH", "GOROOT",
    "MACOSX_DEPLOYMENT_TARGET", "SDKROOT", "DEVELOPER_DIR"
];

#[cfg(target_family = "unix")]
const PATH_SEPARATOR: &str = ":";

#[cfg(target_os = "windows")]
const PATH_SEPARATOR: &str = ";";

/// The errors that occur during creating the build environment.
#[derive(Error, Debug)]
pub enum BuildEnvError {
    #[error("Cannot add {} to build env {variable}: cannot convert PathBuf to string", path.display())]
    PathBufConversion {
        path: PathBuf,
        variable: String,
    },

    #[error("Cannot find tool {0} on the system")]
    ToolNotFound(String),

    #[error("Error while detecting tool on the system")]
    ToolDetectionError(#[from] ToolDetectionError),
}

pub type Result<T> = core::result::Result<T, BuildEnvError>;

/// Holds all the data necessary to build a normalized build environment.
pub struct BuildEnv<'a> {
    prefix_directory: &'a PathBuf,
    dependencies: &'a Vec<&'a InstalledPackageVersion>,
    build_dependencies: Vec<&'a InstalledPackageVersion>,
    build_requirements: &'a Vec<Requirement>,
    register: &'a PackageRegister,
}

impl<'a> TryInto<Environment> for BuildEnv<'a> {
    type Error = BuildEnvError;

    /// Converts the `BuildEnv` struct into a normalized `Environment` struct.
    fn try_into(self) -> Result<Environment> {
        let mut env = Environment::new();

        // TODO: maybe also sandbox TMPDIR variable
        // TODO: maybe use active version path instead of real version path for all dependencies
        env.insert_vars(HashMap::from([
            ("PATH", self.create_path()?),
            ("PKG_CONFIG_PATH", self.create_pkg_config_path()?),
            ("PKG_CONFIG_LIBDIR", "".into()),
            ("CMAKE_PREFIX_PATH", self.create_cmake_prefix_path()?),
            ("ACLOCAL_PATH", self.create_aclocal_path()?),
            ("TZ", "UTC0".into()), // Ensure timezone is the same across all builds
        ]));

        // Strip all vars which should be stripped
        for var in STRIPPED_VARS {
            env.strip_var(*var);
        }

        // Add M4 variable if m4 is a build dependency
        if let Some(m4) = self.build_dependencies.iter().find(|x| *x.package_id.name == "m4") {
            env.insert_var("M4", path_to_string(&m4.install_path, "M4")?);
        }

        // Add requirement specific vars to the build env
        env.expand(Self::create_requirement_environment(self.build_requirements)?);

        // Add macos specific environment variables
        #[cfg(target_os = "macos")]
        {
            env.insert_var("PERL", "/usr/bin/perl");
            env.insert_var("ZERO_AR_DATE", "1"); // Ensure no arbritary timestamps are in builds

            // TODO: add xcode paths
        }

        Ok(env)
    }
}

impl<'a> BuildEnv<'a> {
    /// Creates a new `BuildEnv`.
    pub fn new(
        prefix_directory: &'a PathBuf,
        dependencies: &'a Vec<&'a InstalledPackageVersion>,
        build_dependencies: Vec<&'a InstalledPackageVersion>,
        build_requirements: &'a Vec<Requirement>,
        register: &'a PackageRegister,
    ) -> Self {
        Self {
            prefix_directory,
            dependencies,
            build_dependencies,
            build_requirements,
            register,
        }
    }

    /// Creates the `PATH` for the `Environment`. The path will include the bin directories
    /// of all (build) dependencies and standard Unix system bin paths (if on Unix).
    fn create_path(&self) -> Result<String> {
        let mut parts = Vec::new();

        //TODO: add compiler wrappers to path

        // Add all dependencies to PATH
        let dependencies = self.dependencies.iter().chain(self.build_dependencies.iter());
        for dependency in dependencies {
            let bin_path = dependency.install_path.join("bin");

            // Skip adding if the bin dir does not exist
            if !bin_path.exists() {
                continue;
            }

            parts.push(path_to_string(&bin_path, "PATH")?);
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

    /// Creates the `PKG_CONFIG_PATH` to pkgconfig inside of the lib and share directories of the (build) dependencies.
    /// It also adds the necessary platform specific paths.
    fn create_pkg_config_path(&self) -> Result<String> {
        let mut parts: Vec<String> = Vec::new();

        // Add dependencies to PKG_CONFIG_PATH
        for dependency in self.dependencies {
            let lib_path = dependency.install_path.join("lib").join("pkgconfig");
            let share_path = dependency.install_path.join("share").join("pkgconfig");

            // Add lib dir to PKG_CONFIG_PATH if it exists and is a directory
            if lib_path.is_dir() {
                parts.push(path_to_string(&lib_path, "PKG_CONFIG_PATH")?);
            }

            // Add share dir to PKG_CONFIG_PATH if it exists and is a directory
            if share_path.is_dir() {
                parts.push(path_to_string(&share_path, "PKG_CONFIG_PATH")?);
            }
        }

        // Add macos specific paths
        #[cfg(target_os = "macos")]
        {
            parts.push("/usr/lib/pkgconfig".into());
        }

        Ok(parts.join(PATH_SEPARATOR))
    }

    /// Creates the `CMAKE_PREFIX_PATH` with the (build) dependency install paths.
    fn create_cmake_prefix_path(&self) -> Result<String> {
        let mut parts: Vec<String> = Vec::new();

        // Add non symlinked dependencies to CMAKE_PREFIX_PATH
        for dependency in self.dependencies {
            if let Some(package) = self.register.get_package(&dependency.package_id.name) {
                if package.symlinked {
                    continue;
                }
            }

            parts.push(path_to_string(&dependency.install_path, "CMAKE_PREFIX_PATH")?);
        }

        // Add prefix directory to CMAKE_PREFIX_PATH
        parts.push(path_to_string(self.prefix_directory, "CMAKE_PREFIX_PATH")?);

        Ok(parts.join(PATH_SEPARATOR))
    }

    /// Creates the `ACLOCAL_PATH` from the share/aclocal in each (build) dependency.
    fn create_aclocal_path(&self) -> Result<String> {
        let mut parts: Vec<String> = Vec::new();

        // Add non symlinked dependencies to ACLOCAL_PATH
        for dependency in self.dependencies {
            if let Some(package) = self.register.get_package(&dependency.package_id.name) {
                if package.symlinked {
                    continue;
                }
            }

            let share_path = dependency.install_path.join("share").join("aclocal");

            // Adding the share dir if it exists
            if share_path.exists() {
                parts.push(path_to_string(&share_path, "ACLOCAL_PATH")?);
            }
        }

        // Add prefix directory to ACLOCAL_PATH
        let global_aclocal = self.prefix_directory.join("share").join("aclocal");
        parts.push(path_to_string(&global_aclocal, "ACLOCAL_PATH")?);

        Ok(parts.join(PATH_SEPARATOR))
    }

    /// Creates an environment from the given requirements.
    pub fn create_requirement_environment(requirements: &Vec<Requirement>) -> Result<Environment> {
        let mut environment = Environment::new();

        for requirement in requirements {
            match requirement {
                Requirement::Msvc => {
                    // Detect MSVC toolchain
                    let Some(msvc) = tool_detection::detect_msvc()? else {
                        return Err(BuildEnvError::ToolNotFound("msvc".into()));
                    };

                    // Get arch, skip requirement if arch is `None`.
                    let Some(arch) = msvc.get_vcvarsall_arch(&Target::current()) else {
                        warning!("Tried to load MSVC for an unsupported target, skipping adding to env");
                        continue;
                    };

                    // Add requirement specific environment vars to result
                    environment.insert_vars(HashMap::from([
                        ("PACKIT_VS_PATH", path_to_string(msvc.get_vs_path(), "PACKIT_VS_PATH")?),
                        ("PACKIT_VCVARSALL", path_to_string(&msvc.get_vcvarsall_path(), "PACKIT_VCVARSALL")?),
                        ("PACKIT_VCVARSALL_ARCH", arch.into()),
                        ("PACKIT_MSVC_VERSION", msvc.get_version().to_string()),
                    ]));
                },
            }
        }

        Ok(environment)
    }
}

/// Converts a `PathBuf` to a string.
/// Returns an `Err` if the path cannot be converted.
fn path_to_string(path: &Path, env_var: &str) -> Result<String> {
    match path.to_str() {
        Some(string) => Ok(string.into()),
        None => Err(BuildEnvError::PathBufConversion {
            path: path.to_path_buf(),
            variable: env_var.into(),
        }),
    }
}
