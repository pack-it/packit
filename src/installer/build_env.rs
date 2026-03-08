use std::{collections::HashMap, path::PathBuf};

use crate::{
    cli::display::logging::warning,
    storage::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    utils::env::Environment,
};

// TODO: We should probably also strip tokens from the env
#[rustfmt::skip]
const STRIPPED_VARS: &'static [&'static str] = &[
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

pub struct BuildEnv<'a> {
    prefix_directory: &'a PathBuf,
    dependencies: Vec<&'a InstalledPackageVersion>,
    build_dependencies: Vec<&'a InstalledPackageVersion>,
    register: &'a PackageRegister,
}

impl<'a> Into<Environment> for BuildEnv<'a> {
    fn into(self) -> Environment {
        let mut env = Environment::new();

        // TODO: maybe also sandbox TMPDIR variable
        // TODO: maybe use active version path instead of real version path for all dependencies
        env.insert_vars(HashMap::from([
            ("PATH", self.create_path()),
            ("PKG_CONFIG_PATH", self.create_pkg_config_path()),
            ("PKG_CONFIG_LIBDIR", "".into()),
            ("CMAKE_PREFIX_PATH", self.create_cmake_prefix_path()),
            ("ACLOCAL_PATH", self.create_aclocal_path()),
            ("TZ", "UTC0".into()), // Ensure timezone is the same across all builds
        ]));

        // Strip all vars which should be stripped
        for var in STRIPPED_VARS {
            env.strip_var(*var);
        }

        // Add M4 variable if m4 is a dependency
        if self.build_dependencies.iter().any(|x| x.package_id.name.to_string() == "m4") {
            let m4_path = self.prefix_directory.join("bin").join("m4");
            match m4_path.to_str() {
                Some(path) => drop(env.insert_var("M4", path)),
                None => warning!("Cannot add M4 var to build env: cannot convert PathBuf to string"),
            };
        }

        // Add macos specific environment variables
        #[cfg(target_os = "macos")]
        {
            env.insert_var("PERL", "/usr/bin/perl");
            env.insert_var("ZERO_AR_DATE", "1"); // Ensure no arbritary timestamps are in builds

            // TODO: add xcode paths
        }

        env
    }
}

impl<'a> BuildEnv<'a> {
    pub fn new(
        prefix_directory: &'a PathBuf,
        dependencies: Vec<&'a InstalledPackageVersion>,
        build_dependencies: Vec<&'a InstalledPackageVersion>,
        register: &'a PackageRegister,
    ) -> Self {
        Self {
            prefix_directory,
            dependencies,
            build_dependencies,
            register,
        }
    }

    fn create_path(&self) -> String {
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

            match bin_path.to_str() {
                Some(path) => parts.push(path.into()),
                None => {
                    warning!(
                        "Cannot add dependency {} to build env PATH: cannot convert PathBuf to string",
                        dependency.package_id
                    );
                    continue;
                },
            };
        }

        // Add standard unix system bin paths to PATH
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            parts.append(&mut vec!["/usr/bin", "/bin", "/usr/sbin", "/sbin"].into_iter().map(String::from).collect());
        }

        parts.join(":")
    }

    fn create_pkg_config_path(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add dependencies to PKG_CONFIG_PATH
        for dependency in &self.dependencies {
            let lib_path = dependency.install_path.join("lib").join("pkgconfig");
            let share_path = dependency.install_path.join("share").join("pkgconfig");

            // Add lib dir to PKG_CONFIG_PATH if it exists and is a directory
            if lib_path.is_dir() {
                match lib_path.to_str() {
                    Some(path) => parts.push(path.into()),
                    None => {
                        warning!(
                            "Cannot add dependency {} lib/pkgconfig to build env PKG_CONFIG_PATH: cannot convert PathBuf to string",
                            dependency.package_id
                        );
                        continue;
                    },
                };
            }

            // Add share dir to PKG_CONFIG_PATH if it exists and is a directory
            if share_path.is_dir() {
                match share_path.to_str() {
                    Some(path) => parts.push(path.into()),
                    None => {
                        warning!(
                            "Cannot add dependency {} share/pkgconfig to build env PKG_CONFIG_PATH: cannot convert PathBuf to string",
                            dependency.package_id
                        );
                        continue;
                    },
                };
            }
        }

        // Add macos specific paths
        #[cfg(target_os = "macos")]
        {
            parts.push("/usr/lib/pkgconfig".into());
        }

        parts.join(":")
    }

    fn create_cmake_prefix_path(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add non symlinked dependencies to CMAKE_PREFIX_PATH
        for dependency in &self.dependencies {
            if let Some(package) = self.register.get_package(&dependency.package_id.name) {
                if package.symlinked {
                    continue;
                }
            }

            let path = &dependency.install_path;
            match path.to_str() {
                Some(path) => parts.push(path.into()),
                None => {
                    warning!(
                        "Cannot add dependency {} to build env CMAKE_PREFIX_PATH: cannot convert PathBuf to string",
                        dependency.package_id
                    );
                    continue;
                },
            };
        }

        // Add prefix directory to CMAKE_PREFIX_PATH
        match self.prefix_directory.to_str() {
            Some(path) => parts.push(path.into()),
            None => warning!("Cannot add Packit prefix directory to build env CMAKE_PREFIX_PATH: cannot convert PathBuf to string"),
        };

        parts.join(":")
    }

    fn create_aclocal_path(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add non symlinked dependencies to ACLOCAL_PATH
        for dependency in &self.dependencies {
            if let Some(package) = self.register.get_package(&dependency.package_id.name) {
                if package.symlinked {
                    continue;
                }
            }

            let share_path = dependency.install_path.join("share").join("aclocal");

            // Skip adding if the share dir does not exist
            if !share_path.exists() {
                continue;
            }

            match share_path.to_str() {
                Some(path) => parts.push(path.into()),
                None => {
                    warning!(
                        "Cannot add dependency {} to build env ACLOCAL_PATH: cannot convert PathBuf to string",
                        dependency.package_id
                    );
                    continue;
                },
            };
        }

        // Add prefix directory to ACLOCAL_PATH
        match self.prefix_directory.join("share").join("aclocal").to_str() {
            Some(path) => parts.push(path.into()),
            None => warning!("Cannot add Packit prefix directory to build env ACLOCAL_PATH: cannot convert PathBuf to string"),
        };

        parts.join(":")
    }
}
