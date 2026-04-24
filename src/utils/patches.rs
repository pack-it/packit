// SPDX-License-Identifier: GPL-3.0-only
use std::{fs, path::PathBuf};

use bytes::Bytes;
use diffy::patch_set::FilePatch;
use diffy::{
    binary::BinaryPatch,
    patch_set::{FileOperation, ParseOptions, PatchKind, PatchSet},
};
use thiserror::Error;

use crate::cli::display::logging::warning;
use crate::utils::io;

/// The errors that occur while applying patches.
#[derive(Error, Debug)]
pub enum PatchError {
    #[error("Error while interacting with filesystem")]
    IoError(#[from] std::io::Error),

    #[error("Error while parsing path to UTF-8")]
    PathParseError(#[from] std::str::Utf8Error),

    #[error("Error while parsing patch sets")]
    PatchSetParseError(#[from] diffy::patch_set::PatchSetParseError),

    #[error("Cannot parse binary patch")]
    BinaryPatchParseError(#[from] diffy::binary::BinaryPatchParseError),

    #[error("Cannot apply patch")]
    PatchApplyError(#[from] diffy::ApplyError),
}

pub type Result<T> = std::result::Result<T, PatchError>;

// Applies the given patch to the given directory directory.
pub fn apply_patch(patch: &Bytes, directory: &PathBuf) -> Result<()> {
    let patches = PatchSet::parse_bytes(&patch, ParseOptions::gitdiff());

    for file_patch in patches {
        let file_patch = file_patch?;

        // Remove a and b prefixes from paths that do not come from the `Rename` or `Copy` operation
        let operation = file_patch.operation();
        let operation = match operation {
            FileOperation::Rename { .. } | FileOperation::Copy { .. } => operation,
            _ => &operation.strip_prefix(1),
        };

        handle_file_operation(operation, &file_patch, directory)?;
    }

    Ok(())
}

// Handles the file operation to apply the different patch operations.
fn handle_file_operation(operation: &FileOperation<[u8]>, patch: &FilePatch<[u8]>, directory: &PathBuf) -> Result<()> {
    match operation {
        FileOperation::Create(path) => {
            let path = create_path(directory, path)?;

            apply_path(patch.patch(), None, &path)?;
        },
        FileOperation::Delete(path) => {
            let path = create_path(directory, path)?;
            fs::remove_file(&path)?;
        },
        FileOperation::Modify { original, modified } => {
            let source = create_path(directory, original)?;
            let destination = create_path(directory, modified)?;

            apply_path(patch.patch(), Some(&source), &destination)?;

            // If the source and destination are not the same, the file is also moved.
            if source != destination {
                fs::remove_file(&source)?;
            }
        },
        FileOperation::Rename { from, to } => {
            let source = create_path(directory, from)?;
            let destination = create_path(directory, to)?;

            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::rename(&source, &destination)?;
        },
        FileOperation::Copy { from, to } => {
            let source = create_path(directory, from)?;
            let destination = create_path(directory, to)?;

            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&source, &destination)?;
        },
    }

    Ok(())
}

// Creates a path for the patch by parsing the path from bytes and appending it to the given directory.
fn create_path(directory: &PathBuf, patch_path: &[u8]) -> Result<PathBuf> {
    Ok(directory.join(io::parse_path_from_bytes(patch_path)?))
}

// Applies a text or binary patch to a file. The source argument can be none, to use no original file
fn apply_path(patch: &PatchKind<[u8]>, source: Option<&PathBuf>, destination: &PathBuf) -> Result<()> {
    if matches!(patch, PatchKind::Binary(BinaryPatch::Marker)) {
        warning!("Patch has a binary patch marker, but does not contain any data");
        return Ok(());
    }

    let original = match source {
        Some(source) => fs::read(&source)?,
        None => vec![],
    };

    let patched = match patch {
        PatchKind::Text(patch) => diffy::apply_bytes(&original, &patch)?,
        PatchKind::Binary(patch) => patch.apply(&original)?,
    };

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&destination, patched)?;

    Ok(())
}
