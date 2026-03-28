use std::{collections::VecDeque, fs, path::PathBuf, process::Command};

use lief::{Binary, elf, macho};
use thiserror::Error;

use crate::{
    cli::display::logging::{debug, error},
    config::Config,
    installer::types::PackageId,
    storage::installed_package_version::InstalledPackageVersion,
};

/// The errors that occur during binary operations.
#[derive(Error, Debug)]
pub enum BinaryPatcherError {
    #[error("Binary '{path}' cannot be parsed.")]
    CannotParseBinary {
        path: PathBuf,
    },

    #[error("Binary '{path}' of type {bin_type} is not supported.")]
    UnsupportedBinaryType {
        path: PathBuf,
        bin_type: String,
    },

    #[error("Cannot convert OsString to string")]
    OsStringConversionError,

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, BinaryPatcherError>;

pub struct BinaryPatcher<'a> {
    config: &'a Config,
}

impl<'a> BinaryPatcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn patch_binaries_in(&self, path: &PathBuf, package: &PackageId, dependencies: Vec<&InstalledPackageVersion>) -> Result<()> {
        let metadata = fs::metadata(path)?;

        // If the given path is a file, patch the file directly
        if metadata.is_file() {
            return self.patch_binary(path, package, &dependencies);
        }

        let mut queue = VecDeque::from([path.clone()]);
        while queue.len() != 0 {
            let item = queue.pop_front().expect("Expected an element in the queue");

            for entry in fs::read_dir(item)? {
                let entry = entry?;

                let metadata = entry.metadata()?;

                // If the entry is a directory, add it to the queue
                if metadata.is_dir() {
                    queue.push_back(entry.path());
                }

                // If the entry is a file, try to patch it
                if metadata.is_file() {
                    match self.patch_binary(&entry.path(), package, &dependencies) {
                        Ok(_)
                        | Err(BinaryPatcherError::CannotParseBinary { .. })
                        | Err(BinaryPatcherError::UnsupportedBinaryType { .. }) => (),
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Ok(())
    }

    fn patch_binary(&self, path: &PathBuf, package: &PackageId, dependencies: &Vec<&InstalledPackageVersion>) -> Result<()> {
        match Binary::parse(path) {
            Some(Binary::ELF(binary)) => {
                debug!("Patching ELF binary at '{}'", path.display());
                self.patch_elf(binary, path, package, dependencies)?;
            },
            Some(Binary::MachO(binary)) => {
                debug!("Patching MachO binary at '{}'", path.display());
                self.patch_macho(binary, path, package)?;
            },
            Some(Binary::PE(_binary)) => {
                debug!("Patching PE binary at '{}'", path.display());
                todo!();
            },
            Some(Binary::COFF(_)) => {
                return Err(BinaryPatcherError::UnsupportedBinaryType {
                    path: path.clone(),
                    bin_type: "COFF".into(),
                });
            },
            None => return Err(BinaryPatcherError::CannotParseBinary { path: path.clone() }),
        }

        Ok(())
    }

    fn patch_macho(&self, binary: macho::FatBinary, path: &PathBuf, package: &PackageId) -> Result<()> {
        for mut binary in binary.iter() {
            let mut changed = false;

            for mut library in binary.libraries() {
                let library_path = PathBuf::from(library.name());

                // Check if library lives in the prefix directory
                let prefix = self.config.prefix_directory.join("packages/");
                if library_path.starts_with(&prefix) {
                    debug!("Found Packit dependency: {}", library_path.display());

                    let library_path = match library_path.strip_prefix(&prefix) {
                        Ok(path) => path,
                        Err(_) => {
                            debug!("Cannot remove prefix from dependency path");
                            continue;
                        },
                    };

                    let parts: Vec<_> = library_path.components().collect();
                    if parts.len() < 2 {
                        debug!("Linked dependency path is too short");
                        continue;
                    }

                    let dependency_name = parts[0].as_os_str().to_str().ok_or(BinaryPatcherError::OsStringConversionError)?;
                    let dependency_version = parts[1].as_os_str().to_str().ok_or(BinaryPatcherError::OsStringConversionError)?;

                    // Check if the dependency links to the package itself
                    if dependency_name == *package.name && dependency_version == package.version.to_string() {
                        continue;
                    }

                    let new_prefix = self.config.prefix_directory.join("dependencies").join(package.to_string()).join(dependency_name);
                    let new_suffix: PathBuf = parts.iter().skip(2).collect();
                    let new_path = new_prefix.join(new_suffix);

                    library.set_name(new_path.to_str().ok_or(BinaryPatcherError::OsStringConversionError)?);

                    changed = true;
                }
            }

            if changed {
                debug!("Changed binary {path:?}, writing changes");

                let mut config = macho::builder::Config::default();
                config.linkedit = true;
                binary.write_with_config(path, config);

                // Sign binary
                let path = path.to_str().ok_or(BinaryPatcherError::OsStringConversionError)?;
                match Command::new("/usr/bin/codesign").args(["--sign", "-", "--force", path]).status() {
                    Ok(code) if !code.success() => {
                        error!(msg: "Cannot resign binary, exit code {code}");
                        continue;
                    },
                    Ok(_) => (),
                    Err(e) => {
                        error!(e, "Cannot resign binary {path:?}");
                        continue;
                    },
                };
            }
        }

        Ok(())
    }

    fn patch_elf(
        &self,
        mut binary: elf::Binary,
        path: &PathBuf,
        package: &PackageId,
        dependencies: &Vec<&InstalledPackageVersion>,
    ) -> Result<()> {
        let mut rpaths = Vec::new();

        for entry in binary.dynamic_entries() {
            let library = match entry {
                elf::dynamic::Entries::Library(lib) => lib,
                _ => continue,
            };

            for dependency in dependencies {
                let lib_path = dependency.install_path.join("lib").join(library.name());

                if lib_path.exists() {
                    debug!("Found Packit dependency, adding to rpath");

                    let dependency_path =
                        self.config.prefix_directory.join("dependencies").join(package.to_string()).join(&dependency.package_id.name);
                    rpaths.push(dependency_path);
                }
            }
        }

        // Add rpaths to binary
        if !rpaths.is_empty() {
            debug!("Changed binary {path:?}, writing changes");

            for rpath in rpaths {
                let mut string_path = rpath.to_str().ok_or(BinaryPatcherError::OsStringConversionError)?.to_string();
                if !string_path.ends_with("/") {
                    string_path.push('/');
                }

                binary.add_dynamic_entry(&elf::dynamic::Rpath::new(&string_path));
            }

            let config = elf::builder::Config::default();
            binary.write_with_config(path, config);
        }

        Ok(())
    }
}
