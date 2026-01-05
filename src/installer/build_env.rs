use std::{collections::HashMap, path::PathBuf};

use crate::{cli, installed_packages::InstalledPackage};

pub struct BuildEnv<'a> {
    prefix_directory: &'a PathBuf,
    dependencies: Vec<&'a InstalledPackage>,
    build_dependencies: Vec<&'a InstalledPackage>,
}

impl<'a> BuildEnv<'a> {
    pub fn new(
        prefix_directory: &'a PathBuf,
        dependencies: Vec<&'a InstalledPackage>,
        build_dependencies: Vec<&'a InstalledPackage>,
    ) -> Self {
        Self {
            prefix_directory,
            dependencies,
            build_dependencies,
        }
    }

    pub fn to_hashmap(&self) -> HashMap<&str, String> {
        // TODO: maybe use active version path instead of real version path for all dependencies
        let mut vars = HashMap::from([
            ("PATH", self.create_path()),
            ("PKG_CONFIG_PATH", self.create_pkg_config_path()),
            ("PKG_CONFIG_LIBDIR", "".into()),
            ("CMAKE_PREFIX_PATH", self.create_cmake_prefix_path()),
            ("ACLOCAL_PATH", self.create_aclocal_path()),
        ]);

        // Add M4 variable if m4 is a dependency
        if self.build_dependencies.iter().any(|x| x.name == "m4") {
            let m4_path = self.prefix_directory.join("bin").join("m4");
            match m4_path.to_str() {
                Some(path) => drop(vars.insert("M4", path.into())),
                None => cli::display_warning!("Cannot add M4 var to build env: cannot convert PathBuf to string"),
            };
        }

        // TODO: add xcode paths

        vars
    }

    fn create_path(&self) -> String {
        let mut parts = Vec::new();

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
                    cli::display_warning!(
                        "Cannot add dependency {} to build env PATH: cannot convert PathBuf to string",
                        dependency.name
                    );
                    continue;
                },
            };
        }

        // Add standard system bin paths to PATH
        parts.append(&mut vec!["/usr/bin", "/bin", "/usr/sbin", "/sbin"].into_iter().map(String::from).collect());

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
                        cli::display_warning!(
                            "Cannot add dependency {} lib/pkgconfig to build env PKG_CONFIG_PATH: cannot convert PathBuf to string",
                            dependency.name
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
                        cli::display_warning!(
                            "Cannot add dependency {} share/pkgconfig to build env PKG_CONFIG_PATH: cannot convert PathBuf to string",
                            dependency.name
                        );
                        continue;
                    },
                };
            }
        }

        parts.join(":")
    }

    fn create_cmake_prefix_path(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add non symlinked dependencies to CMAKE_PREFIX_PATH
        for dependency in &self.dependencies {
            if dependency.symlinked {
                continue;
            }

            let path = &dependency.install_path;
            match path.to_str() {
                Some(path) => parts.push(path.into()),
                None => {
                    cli::display_warning!(
                        "Cannot add dependency {} to build env CMAKE_PREFIX_PATH: cannot convert PathBuf to string",
                        dependency.name
                    );
                    continue;
                },
            };
        }

        // Add prefix directory to CMAKE_PREFIX_PATH
        match self.prefix_directory.to_str() {
            Some(path) => parts.push(path.into()),
            None => {
                cli::display_warning!("Cannot add Packit prefix directory to build env CMAKE_PREFIX_PATH: cannot convert PathBuf to string")
            },
        };

        parts.join(":")
    }

    fn create_aclocal_path(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add non symlinked dependencies to ACLOCAL_PATH
        for dependency in &self.dependencies {
            if dependency.symlinked {
                continue;
            }

            let share_path = dependency.install_path.join("share").join("aclocal");

            // Skip adding if the share dir does not exist
            if !share_path.exists() {
                continue;
            }

            match share_path.to_str() {
                Some(path) => parts.push(path.into()),
                None => {
                    cli::display_warning!(
                        "Cannot add dependency {} to build env ACLOCAL_PATH: cannot convert PathBuf to string",
                        dependency.name
                    );
                    continue;
                },
            };
        }

        // Add prefix directory to ACLOCAL_PATH
        match self.prefix_directory.join("share").join("aclocal").to_str() {
            Some(path) => parts.push(path.into()),
            None => {
                cli::display_warning!("Cannot add Packit prefix directory to build env ACLOCAL_PATH: cannot convert PathBuf to string")
            },
        };

        parts.join(":")
    }
}
