// SPDX-License-Identifier: GPL-3.0-only
pub mod error;
pub mod install_tree;
mod installer;
mod options;
pub mod scripts;
mod symlinker;
pub mod types;
pub mod unpack;

pub use self::install_tree::InstallLabel;
pub use self::install_tree::InstallType;

pub use self::installer::Installer;

pub use self::options::InstallerOptions;

pub use self::symlinker::SYMLINK_DIRECTORIES;
pub use self::symlinker::Symlinker;
