// SPDX-License-Identifier: GPL-3.0-only
use std::{
    fs,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use diffy::patch_set::FilePatch;
use diffy::{
    binary::BinaryPatch,
    patch_set::{FileOperation, ParseOptions, PatchKind, PatchSet},
};
use thiserror::Error;

use crate::cli::display::logging::{debug, warning};
use crate::utils::io;

/// The errors that occur while applying patches.
#[derive(Error, Debug)]
pub enum PatchError {
    #[error("Unknown patch format")]
    UnknownPatchFormat,

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

    #[error("Patch is not a UTF-8 string")]
    PatchNotUtf8(std::string::FromUtf8Error),

    #[error("Cannot parse context diff")]
    ContextDiffParseError(#[from] contextdiff_parser::parser::ParserError),
}

pub type Result<T> = std::result::Result<T, PatchError>;

/// Represents the format of a patch.
#[derive(Debug)]
pub enum PatchFormat {
    Context,
    Git,
    Unified,
    Unknown,
}

// Applies the given patch to the given directory directory.
pub fn apply_patch(patch: Bytes, directory: &Path) -> Result<()> {
    // Detect the format of the patch
    let patch_format = detect_patch_format(&patch)?;
    debug!("Detected patch format: {patch_format:?}");

    // Use correct parse options for patch format
    let (patch, options) = match patch_format {
        PatchFormat::Context => {
            let patch_str = String::from_utf8(patch.to_vec()).map_err(PatchError::PatchNotUtf8)?;
            let context_diff = contextdiff_parser::parser::parse_from_str(&patch_str)?;

            debug!("Translating context diff to unified diff");
            let unified_diff = contextdiff_parser::translator::translate_to_unified_diff(context_diff);

            (unified_diff.into_bytes().into(), ParseOptions::unidiff())
        },
        PatchFormat::Git => (patch, ParseOptions::gitdiff()),
        PatchFormat::Unified => (patch, ParseOptions::unidiff()),
        PatchFormat::Unknown => return Err(PatchError::UnknownPatchFormat),
    };

    // Parse patch from bytes
    let patches = PatchSet::parse_bytes(&patch, options);
    for file_patch in patches {
        let file_patch = file_patch?;

        // Remove a and b prefixes from paths
        let operation = file_patch.operation();
        let operation = match contains_git_prefix(operation) {
            true => &operation.strip_prefix(1),
            _ => operation,
        };

        handle_file_operation(operation, &file_patch, directory)?;
    }

    Ok(())
}

// Handles the file operation to apply the different patch operations.
fn handle_file_operation(operation: &FileOperation<[u8]>, patch: &FilePatch<[u8]>, directory: &Path) -> Result<()> {
    match operation {
        FileOperation::Create(path) => {
            let path = create_path(directory, path)?;
            debug!("Creating file '{}'", path.display());

            apply_path(patch.patch(), None, &path)?;
        },
        FileOperation::Delete(path) => {
            let path = create_path(directory, path)?;
            debug!("Deleting file '{}'", path.display());

            fs::remove_file(&path)?;
        },
        FileOperation::Modify { original, modified } => {
            let source = create_path(directory, original)?;
            let destination = create_path(directory, modified)?;
            debug!("Modifying file '{}' to '{}'", source.display(), destination.display());

            apply_path(patch.patch(), Some(&source), &destination)?;

            // If the source and destination are not the same, the file is also moved.
            if source != destination {
                fs::remove_file(&source)?;
            }
        },
        FileOperation::Rename { from, to } => {
            let source = create_path(directory, from)?;
            let destination = create_path(directory, to)?;
            debug!("Renaming file '{}' to '{}'", source.display(), destination.display());

            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::rename(&source, &destination)?;
        },
        FileOperation::Copy { from, to } => {
            let source = create_path(directory, from)?;
            let destination = create_path(directory, to)?;
            debug!("Copying file '{}' to '{}'", source.display(), destination.display());

            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&source, &destination)?;
        },
    }

    Ok(())
}

// Creates a path for the patch by parsing the path from bytes and appending it to the given directory.
fn create_path(directory: &Path, patch_path: &[u8]) -> Result<PathBuf> {
    let path = directory.join(io::parse_path_from_bytes(patch_path)?);
    Ok(io::normalize_path(&path))
}

// Applies a text or binary patch to a file. The source argument can be none, to use no original file
fn apply_path(patch: &PatchKind<[u8]>, source: Option<&PathBuf>, destination: &PathBuf) -> Result<()> {
    if matches!(patch, PatchKind::Binary(BinaryPatch::Marker)) {
        warning!("Patch has a binary patch marker, but does not contain any data");
        return Ok(());
    }

    let original = match source {
        Some(source) => fs::read(source)?,
        None => vec![],
    };

    let patched = match patch {
        PatchKind::Text(patch) => diffy::apply_bytes(&original, patch)?,
        PatchKind::Binary(patch) => patch.apply(&original)?,
    };

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(destination, patched)?;

    Ok(())
}

/// Detect the format of the given patch.
fn detect_patch_format(patch: &Bytes) -> Result<PatchFormat> {
    let mut found_git_header = false;

    let mut found_unified_from = false;
    let mut found_unified_to = false;
    let mut found_unified_hunk = false;

    let mut found_context_from_hunk = false;
    let mut found_context_to_hunk = false;
    let mut found_context_hunk_separator = false;

    for line in patch.split(|&b| b == b'\n') {
        let line = match line.ends_with(b"\r") {
            true => &line[..line.len() - 1],
            false => line,
        };

        if line.starts_with(b"diff --git ") {
            found_git_header = true;
        }

        if line.starts_with(b"--- ") && !line.ends_with(b" ----") {
            found_unified_from = true;
        }
        if line.starts_with(b"+++ ") {
            found_unified_to = true;
        }
        if line.starts_with(b"@@ ") {
            found_unified_hunk = true;
        }

        if line.starts_with(b"*** ") && line.ends_with(b" ****") {
            found_context_from_hunk = true;
        }
        if line.starts_with(b"--- ") && line.ends_with(b" ----") {
            found_context_to_hunk = true;
        }
        if line == b"***************" {
            found_context_hunk_separator = true;
        }
    }

    if found_git_header {
        return Ok(PatchFormat::Git);
    }

    if found_unified_from && found_unified_to && found_unified_hunk {
        return Ok(PatchFormat::Unified);
    }

    if found_context_from_hunk && found_context_to_hunk && found_context_hunk_separator {
        return Ok(PatchFormat::Context);
    }

    Ok(PatchFormat::Unknown)
}

/// Checks if a file operation contains git `a/` and `b/` prefixes.
fn contains_git_prefix(operation: &FileOperation<[u8]>) -> bool {
    match operation {
        FileOperation::Delete(path) if path.starts_with(b"a/") => true,
        FileOperation::Create(path) if path.starts_with(b"b/") => true,
        FileOperation::Modify { original, modified } if original.starts_with(b"a/") && modified.starts_with(b"b/") => true,
        FileOperation::Rename { from, to } if from.starts_with(b"a/") && to.starts_with(b"b/") => true,
        FileOperation::Copy { from, to } if from.starts_with(b"a/") && to.starts_with(b"b/") => true,
        _ => false,
    }
}
