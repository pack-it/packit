use std::{collections::VecDeque, fs, path::PathBuf};

use lief::{Binary, elf, macho};
use thiserror::Error;

use crate::{cli::display::logging::debug, config::Config};

/// The errors that occur during binary operations.
#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("Binary '{path}' cannot be parsed.")]
    CannotParseBinary {
        path: PathBuf,
    },

    #[error("Binary '{path}' of type {bin_type} is not supported.")]
    UnsupportedBinaryType {
        path: PathBuf,
        bin_type: String,
    },

    #[error("Error while interacting with filesystem")]
    IOError(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, BinaryError>;

pub struct BinaryPatcher<'a> {
    config: &'a Config,
}

impl<'a> BinaryPatcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn patch_binaries_in(&self, path: &PathBuf) -> Result<()> {
        let metadata = fs::metadata(path)?;

        // If the given path is a file, patch the file directly
        if metadata.is_file() {
            return self.patch_binary(path);
        }

        let mut queue = VecDeque::from([path.clone()]);
        while queue.len() != 0 {
            let item = queue.pop_front().expect("Expected an element in the queue");
            debug!("Searching for binaries in '{}'", item.display());

            for entry in fs::read_dir(item)? {
                let entry = entry?;

                let metadata = entry.metadata()?;

                // If the entry is a directory, add it to the queue
                if metadata.is_dir() {
                    queue.push_back(entry.path());
                }

                // If the entry is a file, try to patch it
                if metadata.is_file() {
                    match self.patch_binary(&entry.path()) {
                        Ok(_) | Err(BinaryError::CannotParseBinary { .. }) | Err(BinaryError::UnsupportedBinaryType { .. }) => (),
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Ok(())
    }

    fn patch_binary(&self, path: &PathBuf) -> Result<()> {
        match Binary::parse(path) {
            Some(Binary::ELF(binary)) => {
                debug!("Patching ELF binary at '{}'", path.display());
                self.patch_elf(binary, path)?;
            },
            Some(Binary::MachO(binary)) => {
                debug!("Patching MachO binary at '{}'", path.display());
                self.patch_macho(binary, path)?;
            },
            Some(Binary::PE(_binary)) => {
                debug!("Patching PE binary at '{}'", path.display());
                todo!();
            },
            Some(Binary::COFF(_)) => {
                return Err(BinaryError::UnsupportedBinaryType {
                    path: path.clone(),
                    bin_type: "COFF".into(),
                });
            },
            None => return Err(BinaryError::CannotParseBinary { path: path.clone() }),
        }

        Ok(())
    }

    fn patch_macho(&self, binary: macho::FatBinary, path: &PathBuf) -> Result<()> {
        for mut binary in binary.iter() {
            let mut changed = false;

            for library in binary.libraries() {
                let library_path = PathBuf::from(library.name());

                // Check if library lives in the prefix directory
                if library_path.starts_with(&self.config.prefix_directory) {
                    debug!("Found Packit dependency: {}", library_path.display());

                    // TODO: split out package name and link to correct directory
                    // TODO: check if the library links to a path outside of the current package
                    // changed = true;
                }
            }

            if changed {
                let mut config = macho::builder::Config::default();
                config.linkedit = true;
                binary.write_with_config(path, config);
            }
        }

        Ok(())
    }

    fn patch_elf(&self, binary: elf::Binary, path: &PathBuf) -> Result<()> {
        todo!();
        Ok(())
    }
}
