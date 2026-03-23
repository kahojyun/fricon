//! Archive import/export helpers for dataset portability.
//!
//! # Ownership
//!
//! This module owns the archive format (tar+zstd with `metadata.json` +
//! `data/data_chunk_*.arrow` entries) and the filesystem-side import
//! workflow. Database writes and event publishing stay in higher layers
//! ([`DatasetCatalogService`](super::catalog::DatasetCatalogService)).
//!
//! # Import workflow (caller-driven)
//!
//! The import flow is intentionally split into discrete steps so callers
//! can coordinate filesystem changes with repository updates:
//!
//! 1. [`preview_import`] — read metadata, detect uuid conflicts.
//! 2. [`stage_import`] — extract archive into a temp sibling directory.
//! 3. [`promote_staged_import`] — move staged data into the live location
//!    (backing up the existing directory when force-replacing).
//! 4. Caller commits repository changes.
//! 5. [`finalize_promoted_import`] / [`rollback_promoted_import`] — depending
//!    on whether the repository commit succeeded.
//!
//! # Archive format
//!
//! ```text
//! metadata.json            ← ExportedMetadata (JSON)
//! data/data_chunk_0.arrow  ← Arrow IPC chunk files
//! data/data_chunk_1.arrow
//! …
//! ```
//!
//! # Extension notes
//!
//! - Adding a metadata field to [`ExportedMetadata`] requires updating
//!   [`compute_diffs`] and the repository import methods in
//!   `database::dataset`.
//! - The archive extraction allowlist in [`extract_archive`] only unpacks
//!   `data/data_chunk_*.arrow` entries. New file types need an explicit entry
//!   in the allowlist.

use std::{
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::dataset::model::{DatasetMetadata, DatasetStatus};

/// Entry name for the metadata file inside the archive.
const METADATA_ENTRY: &str = "metadata.json";
/// Prefix used for data chunk files inside the archive.
const DATA_PREFIX: &str = "data/";
const MAX_ARCHIVE_NAME_CHARS: usize = 64;
const FALLBACK_ARCHIVE_NAME: &str = "dataset";

#[derive(Debug, Error)]
pub enum PortabilityError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Zstd error: {0}")]
    Zstd(io::Error),
    #[error("Archive does not contain a metadata entry")]
    MissingMetadata,
    #[error("Dataset already exists (uuid {uid}); use force=true to overwrite")]
    UuidConflict { uid: Uuid },
    #[error(
        "Dataset storage directory already exists for uuid {uid}; use force=true to overwrite the \
         on-disk data"
    )]
    FilesystemConflict { uid: Uuid },
}

/// Metadata stored inside a portable archive.
///
/// Internal database ids are intentionally excluded so imports can recreate or
/// replace records without trusting archive-local ids.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedMetadata {
    pub uid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub status: DatasetStatus,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl ExportedMetadata {
    /// Build from a live [`DatasetMetadata`].
    #[must_use]
    pub fn from_metadata(m: &DatasetMetadata) -> Self {
        Self {
            uid: m.uid,
            name: m.name.clone(),
            description: m.description.clone(),
            favorite: m.favorite,
            status: m.status,
            created_at: m.created_at,
            tags: m.tags.clone(),
        }
    }
}

/// A single field difference between the existing dataset and the archive.
///
/// Used by [`ImportPreview`] to show the user what would change on import.
#[derive(Debug, Clone)]
pub struct FieldDiff {
    pub field: String,
    pub existing_value: String,
    pub incoming_value: String,
}

/// Information about a uuid conflict found during import preview.
#[derive(Debug, Clone)]
pub struct ImportConflict {
    pub existing: ExportedMetadata,
    pub diffs: Vec<FieldDiff>,
}

/// Result of inspecting an archive before actually importing it.
#[derive(Debug, Clone)]
pub struct ImportPreview {
    pub metadata: ExportedMetadata,
    /// Present only when a dataset with the same uuid already exists.
    pub conflict: Option<ImportConflict>,
}

/// Staged import data held between [`stage_import`] and
/// [`promote_staged_import`].
///
/// The caller must eventually call either [`promote_staged_import`] or
/// [`discard_staged_import`] to clean up the temporary directory.
#[derive(Debug, Clone)]
pub struct StagedImport {
    /// Metadata read from the archive before extraction.
    pub metadata: ExportedMetadata,
    /// Temporary sibling directory that holds extracted files until promotion.
    pub staging_dir: PathBuf,
}

/// Export a single dataset to a `.tar.zst` archive inside `output_dir`.
///
/// The archive contains `metadata.json` plus `data_chunk_*.arrow` files copied
/// from `dataset_dir`.
///
/// The archive name is `{created_at:%Y%m%d_%H%M%S}_{sanitized_name}.tar.zst`.
///
/// Returns the path of the created archive file.
///
/// # Errors
///
/// Returns [`PortabilityError`] on I/O, JSON, or zstd failures.
pub fn export_dataset(
    metadata: &DatasetMetadata,
    dataset_dir: &Path,
    output_dir: &Path,
) -> Result<PathBuf, PortabilityError> {
    fs::create_dir_all(output_dir)?;

    let (out_file, archive_path) = create_archive_file(output_dir, metadata)?;
    let zstd_encoder = zstd::Encoder::new(out_file, 0).map_err(PortabilityError::Zstd)?;
    let mut tar = tar::Builder::new(zstd_encoder);

    // --- metadata.json ---
    let exported = ExportedMetadata::from_metadata(metadata);
    let json_bytes = serde_json::to_vec_pretty(&exported)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(json_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, METADATA_ENTRY, json_bytes.as_slice())?;

    // --- data chunk files ---
    if dataset_dir.is_dir() {
        let mut entries = fs::read_dir(dataset_dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.retain(|e| {
            let path = e.path();
            let is_data_chunk = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("data_chunk_"));
            let is_arrow = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("arrow"));
            is_data_chunk && is_arrow
        });
        // Sort for deterministic archive order.
        entries.sort_by_key(std::fs::DirEntry::file_name);

        for entry in entries {
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let archive_entry_name = format!("{}{}", DATA_PREFIX, file_name.to_string_lossy());
            tar.append_path_with_name(&entry_path, &archive_entry_name)?;
        }
    }

    // Finish the zstd stream properly.
    let zstd_encoder = tar.into_inner()?;
    zstd_encoder.finish().map_err(PortabilityError::Zstd)?;

    Ok(archive_path)
}

/// Read archive metadata without extracting data files.
///
/// If `existing` is provided, the result includes field-level diffs between the
/// live dataset metadata and the incoming archive metadata.
///
/// # Errors
///
/// Returns [`PortabilityError`] on I/O, JSON, or archive parsing failures.
pub fn preview_import(
    archive_path: &Path,
    existing: Option<&DatasetMetadata>,
) -> Result<ImportPreview, PortabilityError> {
    let metadata = read_metadata_from_archive(archive_path)?;

    let conflict = existing.map(|ex| {
        let existing_exported = ExportedMetadata::from_metadata(ex);
        let diffs = compute_diffs(&existing_exported, &metadata);
        ImportConflict {
            existing: existing_exported,
            diffs,
        }
    });

    Ok(ImportPreview { metadata, conflict })
}

/// Extract an archive into a temporary sibling directory of `dest_dir`.
///
/// This does not modify `dest_dir`. Callers can safely inspect the staged
/// result and then either promote or discard it.
///
/// # Errors
///
/// Returns [`PortabilityError`] on I/O, JSON, or archive failures.
pub fn stage_import(
    archive_path: &Path,
    dest_dir: &Path,
) -> Result<StagedImport, PortabilityError> {
    let metadata = read_metadata_from_archive(archive_path)?;
    let parent_dir = dest_dir.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "dataset destination must have a parent directory",
        )
    })?;
    fs::create_dir_all(parent_dir)?;
    let staging_dir = unique_sibling_dir(parent_dir, "import-staging");
    fs::create_dir_all(&staging_dir)?;
    extract_archive(archive_path, &staging_dir).inspect_err(|_| {
        let _ = fs::remove_dir_all(&staging_dir);
    })?;
    Ok(StagedImport {
        metadata,
        staging_dir,
    })
}

/// Promote a staged import into the live dataset location.
///
/// If `dest_dir` already exists and `force` is `true`, the existing live
/// dataset directory is moved aside first and the backup path is returned so
/// the caller can later finalize or roll back the replacement.
pub fn promote_staged_import(
    staged_dir: &Path,
    dest_dir: &Path,
    force: bool,
    uid: Uuid,
) -> Result<Option<PathBuf>, PortabilityError> {
    if !dest_dir.exists() {
        fs::rename(staged_dir, dest_dir)?;
        return Ok(None);
    }
    if !force {
        return Err(PortabilityError::FilesystemConflict { uid });
    }

    let parent_dir = dest_dir.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "dataset destination must have a parent directory",
        )
    })?;
    let backup_dir = unique_sibling_dir(parent_dir, "import-backup");
    fs::rename(dest_dir, &backup_dir)?;
    if let Err(error) = fs::rename(staged_dir, dest_dir) {
        let _ = fs::rename(&backup_dir, dest_dir);
        return Err(PortabilityError::Io(error));
    }

    Ok(Some(backup_dir))
}

/// Roll back a promoted import after a later failure.
///
/// If `backup_dir` is present, the previous live dataset payload is restored.
/// Otherwise the newly promoted live directory is removed.
pub fn rollback_promoted_import(
    dest_dir: &Path,
    backup_dir: Option<&Path>,
) -> Result<(), PortabilityError> {
    remove_dir_if_exists(dest_dir)?;
    if let Some(backup_dir) = backup_dir
        && backup_dir.exists()
    {
        fs::rename(backup_dir, dest_dir)?;
    }
    Ok(())
}

/// Finalize a promoted import by deleting any displaced live payload.
pub fn finalize_promoted_import(backup_dir: Option<&Path>) -> Result<(), PortabilityError> {
    if let Some(backup_dir) = backup_dir {
        remove_dir_if_exists(backup_dir)?;
    }
    Ok(())
}

/// Remove a staged import directory if it still exists.
pub fn discard_staged_import(staged_dir: &Path) -> Result<(), PortabilityError> {
    remove_dir_if_exists(staged_dir)
}

/// Build the user-friendly archive filename.
fn build_archive_name(created_at: &DateTime<Utc>, uid: Uuid, name: &str) -> String {
    let timestamp = created_at.format("%Y%m%d_%H%M%S");
    let safe_name = sanitize_name(name);
    let uid_fragment = uid.simple().to_string();
    format!("{timestamp}_{}_{safe_name}.tar.zst", &uid_fragment[..8])
}

/// Create a new archive file path without overwriting an existing export.
fn create_archive_file(
    output_dir: &Path,
    metadata: &DatasetMetadata,
) -> Result<(File, PathBuf), PortabilityError> {
    let base_name = build_archive_name(&metadata.created_at, metadata.uid, &metadata.name);
    let base_stem = base_name.trim_end_matches(".tar.zst");

    for attempt in 0.. {
        let archive_name = if attempt == 0 {
            base_name.clone()
        } else {
            format!("{base_stem}_{attempt}.tar.zst")
        };
        let archive_path = output_dir.join(archive_name);
        match File::options()
            .write(true)
            .create_new(true)
            .open(&archive_path)
        {
            Ok(file) => return Ok((file, archive_path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(PortabilityError::Io(error)),
        }
    }

    unreachable!("archive file creation should always return or error")
}

/// Sanitize a dataset name for cross-platform archive filenames.
///
/// Unicode letters and numbers are preserved. Whitespace and other unsafe
/// characters collapse into `_`. ASCII `-` is preserved only when it sits
/// between alphanumeric characters, so separator runs normalize consistently.
fn sanitize_name(name: &str) -> String {
    let trimmed = name.trim_matches(|c: char| c == '.' || c.is_whitespace());
    let chars: Vec<char> = trimmed.chars().collect();
    let mut sanitized = String::with_capacity(chars.len().min(MAX_ARCHIVE_NAME_CHARS));
    let mut sanitized_len = 0usize;
    let mut pending_separator = false;

    for (idx, c) in chars.iter().copied().enumerate() {
        if c == '\0' || c.is_ascii_control() {
            continue;
        }

        if c.is_alphanumeric() {
            if pending_separator && !sanitized.is_empty() && sanitized_len < MAX_ARCHIVE_NAME_CHARS
            {
                sanitized.push('_');
                sanitized_len += 1;
            }
            pending_separator = false;

            if sanitized_len == MAX_ARCHIVE_NAME_CHARS {
                break;
            }
            sanitized.push(c);
            sanitized_len += 1;
            continue;
        }

        if is_preserved_hyphen(&chars, idx, pending_separator) {
            if sanitized_len == MAX_ARCHIVE_NAME_CHARS {
                break;
            }
            sanitized.push('-');
            sanitized_len += 1;
            pending_separator = false;
            continue;
        }

        pending_separator = !sanitized.is_empty();
    }

    if sanitized.is_empty() {
        FALLBACK_ARCHIVE_NAME.to_string()
    } else {
        sanitized
    }
}

fn is_preserved_hyphen(chars: &[char], idx: usize, pending_separator: bool) -> bool {
    if pending_separator || chars[idx] != '-' {
        return false;
    }

    let prev_is_word = idx
        .checked_sub(1)
        .and_then(|prev| chars.get(prev))
        .is_some_and(|c| c.is_alphanumeric());
    let next_is_word = chars.get(idx + 1).is_some_and(|c| c.is_alphanumeric());

    prev_is_word && next_is_word
}

/// Create a hidden unique sibling directory name under `parent_dir`.
fn unique_sibling_dir(parent_dir: &Path, prefix: &str) -> PathBuf {
    parent_dir.join(format!(".{prefix}-{}", Uuid::new_v4().simple()))
}

/// Remove a directory tree when it exists, ignoring missing paths.
fn remove_dir_if_exists(path: &Path) -> Result<(), PortabilityError> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(PortabilityError::Io(error)),
    }
}

/// Open a tar+zstd archive and read only the `metadata.json` entry.
fn read_metadata_from_archive(archive_path: &Path) -> Result<ExportedMetadata, PortabilityError> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let decoder = zstd::Decoder::new(reader).map_err(PortabilityError::Zstd)?;
    let mut tar = tar::Archive::new(decoder);

    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if path.to_str() == Some(METADATA_ENTRY) {
            let mut buf = String::new();
            entry.read_to_string(&mut buf)?;
            let m: ExportedMetadata = serde_json::from_str(&buf)?;
            return Ok(m);
        }
    }

    Err(PortabilityError::MissingMetadata)
}

/// Extract all entries from a tar+zstd archive into `dest_dir`.
///
/// Only `data/data_chunk_*.arrow` entries are extracted.
fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), PortabilityError> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let decoder = zstd::Decoder::new(reader).map_err(PortabilityError::Zstd)?;
    let mut tar = tar::Archive::new(decoder);

    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let Some(path_str) = path.to_str() else {
            continue;
        };

        let dest_file_path = if let Some(file_name) = path_str.strip_prefix(DATA_PREFIX) {
            if is_safe_archive_chunk_name(file_name) {
                dest_dir.join(file_name)
            } else {
                continue;
            }
        } else {
            continue;
        };

        let mut dest_file = File::create(&dest_file_path)?;
        io::copy(&mut entry, &mut dest_file)?;
    }

    Ok(())
}

/// Validate that a data chunk filename is safe to extract.
///
/// Guards against path traversal: only `data_chunk_*.arrow` basenames
/// without path separators are allowed.
fn is_safe_archive_chunk_name(file_name: &str) -> bool {
    let is_data_chunk = file_name.starts_with("data_chunk_");
    let is_arrow = Path::new(file_name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("arrow"));
    let has_path_separators = file_name.contains('/') || file_name.contains('\\');

    is_data_chunk && is_arrow && !has_path_separators
}

/// Compute field-level diffs between `existing` and `incoming` metadata.
fn compute_diffs(existing: &ExportedMetadata, incoming: &ExportedMetadata) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    if existing.name != incoming.name {
        diffs.push(FieldDiff {
            field: "name".to_string(),
            existing_value: existing.name.clone(),
            incoming_value: incoming.name.clone(),
        });
    }
    if existing.description != incoming.description {
        diffs.push(FieldDiff {
            field: "description".to_string(),
            existing_value: existing.description.clone(),
            incoming_value: incoming.description.clone(),
        });
    }
    if existing.favorite != incoming.favorite {
        diffs.push(FieldDiff {
            field: "favorite".to_string(),
            existing_value: existing.favorite.to_string(),
            incoming_value: incoming.favorite.to_string(),
        });
    }
    if existing.status != incoming.status {
        diffs.push(FieldDiff {
            field: "status".to_string(),
            existing_value: format!("{:?}", existing.status),
            incoming_value: format!("{:?}", incoming.status),
        });
    }
    if existing.created_at != incoming.created_at {
        diffs.push(FieldDiff {
            field: "created_at".to_string(),
            existing_value: existing.created_at.to_rfc3339(),
            incoming_value: incoming.created_at.to_rfc3339(),
        });
    }
    let mut ex_tags = existing.tags.clone();
    let mut in_tags = incoming.tags.clone();
    ex_tags.sort_unstable();
    in_tags.sort_unstable();
    if ex_tags != in_tags {
        diffs.push(FieldDiff {
            field: "tags".to_string(),
            existing_value: ex_tags.join(", "),
            incoming_value: in_tags.join(", "),
        });
    }

    diffs
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::dataset::model::{DatasetMetadata, DatasetStatus};

    fn make_metadata(uid: Uuid, name: &str) -> DatasetMetadata {
        DatasetMetadata {
            uid,
            name: name.to_string(),
            description: "test description".to_string(),
            favorite: true,
            status: DatasetStatus::Completed,
            created_at: Utc::now(),
            trashed_at: None,
            deleted_at: None,
            tags: vec!["alpha".to_string(), "beta".to_string()],
        }
    }

    fn dummy_chunk_file(dir: &Path) {
        let chunk_path = dir.join("data_chunk_0.arrow");
        fs::write(chunk_path, b"ARROW_DUMMY_DATA").expect("write chunk");
    }

    // ── export ────────────────────────────────────────────────────────────────

    #[test]
    fn export_creates_archive_with_correct_name() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");
        dummy_chunk_file(&ds_dir);

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "my-dataset");
        let output_dir = tmp.path().join("exports");

        let archive = export_dataset(&meta, &ds_dir, &output_dir).expect("export");

        assert!(archive.exists(), "archive file should exist");
        let name = archive.file_name().and_then(|n| n.to_str()).expect("name");
        assert!(name.ends_with(".tar.zst"), "should end with .tar.zst");
        assert!(
            name.contains("my-dataset"),
            "name should contain dataset name"
        );
        assert!(name.contains(&uid.simple().to_string()[..8]));
    }

    #[test]
    fn export_uses_unique_archive_names_for_repeated_exports() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");
        dummy_chunk_file(&ds_dir);

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "duplicate-name");
        let output_dir = tmp.path().join("exports");

        let first = export_dataset(&meta, &ds_dir, &output_dir).expect("first export");
        let second = export_dataset(&meta, &ds_dir, &output_dir).expect("second export");

        assert_ne!(first, second, "repeated exports should not overwrite");
        assert!(first.exists(), "first archive should still exist");
        assert!(second.exists(), "second archive should exist");
    }

    #[test]
    fn exported_metadata_excludes_id_field() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "noidstest");
        let output_dir = tmp.path().join("exports");
        let archive = export_dataset(&meta, &ds_dir, &output_dir).expect("export");

        // Reopen and read metadata.json raw.
        let file = fs::File::open(&archive).expect("open archive");
        let decoder = zstd::Decoder::new(std::io::BufReader::new(file)).expect("decoder");
        let mut tar = tar::Archive::new(decoder);
        for entry in tar.entries().expect("entries") {
            let mut entry = entry.expect("entry");
            let path = entry.path().expect("path").into_owned();
            if path.to_str() == Some(METADATA_ENTRY) {
                let mut buf = String::new();
                entry.read_to_string(&mut buf).expect("read");
                // JSON must not contain any "id" key at top level
                let val: serde_json::Value = serde_json::from_str(&buf).expect("parse");
                assert!(
                    val.get("id").is_none(),
                    "exported metadata must not contain 'id' field"
                );
                return;
            }
        }
        panic!("metadata.json not found in archive");
    }

    // ── preview ───────────────────────────────────────────────────────────────

    #[test]
    fn preview_reads_metadata_from_archive() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "preview-test");
        let output_dir = tmp.path().join("exports");
        let archive = export_dataset(&meta, &ds_dir, &output_dir).expect("export");

        let preview = preview_import(&archive, None).expect("preview");

        assert_eq!(preview.metadata.uid, uid);
        assert_eq!(preview.metadata.name, "preview-test");
        assert!(preview.conflict.is_none());
    }

    #[test]
    fn preview_detects_conflict_with_name_diff() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "incoming-name");
        let output_dir = tmp.path().join("exports");
        let archive = export_dataset(&meta, &ds_dir, &output_dir).expect("export");

        let existing = make_metadata(uid, "existing-name");
        let preview = preview_import(&archive, Some(&existing)).expect("preview");

        assert!(preview.conflict.is_some());
        let conflict = preview.conflict.unwrap();
        assert!(
            conflict.diffs.iter().any(|d| d.field == "name"),
            "should see a name diff"
        );
    }

    // ── import staging / promotion ───────────────────────────────────────────

    #[test]
    fn stage_import_extracts_data_files_to_staging_dir() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");
        dummy_chunk_file(&ds_dir);

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "import-test");
        let output_dir = tmp.path().join("exports");
        let archive = export_dataset(&meta, &ds_dir, &output_dir).expect("export");

        let dest = tmp.path().join("aa").join(uid.to_string());
        let staged = stage_import(&archive, &dest).expect("stage import");

        assert_eq!(staged.metadata.uid, uid);
        assert!(
            staged.staging_dir.join("data_chunk_0.arrow").exists(),
            "chunk file should be extracted to staging"
        );
        assert!(
            !staged.staging_dir.join("metadata.json").exists(),
            "metadata file should not be extracted into staging"
        );
        assert!(
            !dest.exists(),
            "live dataset directory should not be created during staging"
        );
    }

    #[test]
    fn stage_import_ignores_nested_data_entry_paths() {
        let tmp = TempDir::new().expect("temp dir");
        let archive = tmp.path().join("nested.tar.zst");
        let exported = ExportedMetadata {
            uid: Uuid::new_v4(),
            name: "nested".to_string(),
            description: "nested".to_string(),
            favorite: false,
            status: DatasetStatus::Completed,
            created_at: Utc::now(),
            tags: Vec::new(),
        };

        let out_file = File::create(&archive).expect("archive file");
        let zstd_encoder = zstd::Encoder::new(out_file, 0).expect("encoder");
        let mut tar = tar::Builder::new(zstd_encoder);

        let json_bytes = serde_json::to_vec_pretty(&exported).expect("metadata");
        let mut metadata_header = tar::Header::new_gnu();
        metadata_header.set_size(json_bytes.len() as u64);
        metadata_header.set_mode(0o644);
        metadata_header.set_cksum();
        tar.append_data(&mut metadata_header, METADATA_ENTRY, json_bytes.as_slice())
            .expect("metadata entry");

        let good_bytes = b"GOOD";
        let mut good_header = tar::Header::new_gnu();
        good_header.set_size(good_bytes.len() as u64);
        good_header.set_mode(0o644);
        good_header.set_cksum();
        tar.append_data(
            &mut good_header,
            "data/data_chunk_0.arrow",
            good_bytes.as_slice(),
        )
        .expect("good entry");

        let nested_bytes = b"BAD";
        let mut nested_header = tar::Header::new_gnu();
        nested_header.set_size(nested_bytes.len() as u64);
        nested_header.set_mode(0o644);
        nested_header.set_cksum();
        tar.append_data(
            &mut nested_header,
            "data/data_chunk_0.arrow/evil.arrow",
            nested_bytes.as_slice(),
        )
        .expect("nested entry");

        let zstd_encoder = tar.into_inner().expect("tar finalize");
        zstd_encoder.finish().expect("zstd finish");

        let dest = tmp.path().join("cc").join(exported.uid.to_string());
        let staged = stage_import(&archive, &dest).expect("stage import");

        assert!(
            staged.staging_dir.join("data_chunk_0.arrow").exists(),
            "valid chunk should be extracted"
        );
        assert!(
            !staged.staging_dir.join("evil.arrow").exists(),
            "nested chunk path should be ignored"
        );
    }

    #[test]
    fn stage_import_failure_leaves_existing_live_dir_untouched() {
        let tmp = TempDir::new().expect("temp dir");
        let archive = tmp.path().join("broken.tar.zst");
        fs::write(&archive, b"not a real archive").expect("write archive");

        let dest = tmp.path().join("bb").join(Uuid::new_v4().to_string());
        fs::create_dir_all(&dest).expect("create live dir");
        fs::write(dest.join("old.arrow"), b"OLD").expect("write live payload");

        let result = stage_import(&archive, &dest);

        assert!(result.is_err(), "broken archive should fail staging");
        assert!(
            dest.join("old.arrow").exists(),
            "existing live payload should remain untouched"
        );
    }

    #[test]
    fn promote_without_force_returns_filesystem_conflict_when_live_dir_exists() {
        let tmp = TempDir::new().expect("temp dir");
        let staged = tmp.path().join("stage");
        fs::create_dir_all(&staged).expect("create staged dir");
        fs::write(staged.join("data_chunk_0.arrow"), b"NEW").expect("write staged payload");

        let dest = tmp.path().join("live");
        fs::create_dir_all(&dest).expect("create live dir");
        fs::write(dest.join("old.arrow"), b"OLD").expect("write live payload");

        let uid = Uuid::new_v4();
        let result = promote_staged_import(&staged, &dest, false, uid);

        assert!(
            matches!(
                result,
                Err(PortabilityError::FilesystemConflict {
                    uid: conflict_uid,
                }) if conflict_uid == uid
            ),
            "should return filesystem conflict error"
        );
        assert!(
            staged.exists(),
            "staged dir should remain for caller cleanup"
        );
        assert!(
            dest.join("old.arrow").exists(),
            "existing live payload should remain untouched"
        );
    }

    #[test]
    fn rollback_promoted_overwrite_restores_previous_live_dir() {
        let tmp = TempDir::new().expect("temp dir");
        let staged = tmp.path().join("stage");
        fs::create_dir_all(&staged).expect("create staged dir");
        fs::write(staged.join("data_chunk_0.arrow"), b"NEW").expect("write staged payload");

        let dest = tmp.path().join("live");
        fs::create_dir_all(&dest).expect("create live dir");
        fs::write(dest.join("old.arrow"), b"OLD").expect("write live payload");

        let backup_dir =
            promote_staged_import(&staged, &dest, true, Uuid::new_v4()).expect("promote");
        assert!(
            dest.join("data_chunk_0.arrow").exists(),
            "new payload should be live after promotion"
        );

        rollback_promoted_import(&dest, backup_dir.as_deref()).expect("rollback");

        assert!(
            dest.join("old.arrow").exists(),
            "rollback should restore the original live payload"
        );
        assert!(
            !dest.join("data_chunk_0.arrow").exists(),
            "rollback should remove the promoted payload"
        );
    }

    #[test]
    fn rollback_promoted_fresh_import_removes_new_live_dir() {
        let tmp = TempDir::new().expect("temp dir");
        let staged = tmp.path().join("stage");
        fs::create_dir_all(&staged).expect("create staged dir");
        fs::write(staged.join("data_chunk_0.arrow"), b"NEW").expect("write staged payload");

        let dest = tmp.path().join("live");
        promote_staged_import(&staged, &dest, false, Uuid::new_v4()).expect("promote");
        rollback_promoted_import(&dest, None).expect("rollback");

        assert!(
            !dest.exists(),
            "rollback should remove a freshly promoted live directory"
        );
    }

    // ── sanitize_name ─────────────────────────────────────────────────────────

    #[test]
    fn sanitize_preserves_cjk_letters() {
        let result = sanitize_name("中文数据集");
        assert_eq!(result, "中文数据集");
    }

    #[test]
    fn sanitize_preserves_internal_hyphens_in_mixed_unicode_names() {
        let result = sanitize_name("dataset-中文-01");
        assert_eq!(result, "dataset-中文-01");
    }

    #[test]
    fn sanitize_replaces_unsafe_chars_and_collapses_separators() {
        let result = sanitize_name("bad<name>:a\"b/c\\d|e?f*g");
        assert_eq!(result, "bad_name_a_b_c_d_e_f_g");
    }

    #[test]
    fn sanitize_normalizes_whitespace_runs() {
        let result = sanitize_name("hello \t \n world");
        assert_eq!(result, "hello_world");
    }

    #[test]
    fn sanitize_falls_back_when_name_becomes_empty() {
        let result = sanitize_name("  ./\\***??  ");
        assert_eq!(result, "dataset");
    }

    #[test]
    fn sanitize_removes_leading_and_trailing_dots_and_spaces() {
        let result = sanitize_name("  ..hello world..  ");
        assert_eq!(result, "hello_world");
    }

    #[test]
    fn sanitize_truncates_long_names() {
        let long_name = "中".repeat(100);
        assert_eq!(sanitize_name(&long_name).chars().count(), 64);
    }

    #[test]
    fn export_preserves_unicode_name_in_archive_filename() {
        let tmp = TempDir::new().expect("temp dir");
        let ds_dir = tmp.path().join("dataset");
        fs::create_dir_all(&ds_dir).expect("create dataset dir");
        dummy_chunk_file(&ds_dir);

        let uid = Uuid::new_v4();
        let meta = make_metadata(uid, "dataset-中文-01");
        let output_dir = tmp.path().join("exports");

        let archive = export_dataset(&meta, &ds_dir, &output_dir).expect("export");
        let name = archive.file_name().and_then(|n| n.to_str()).expect("name");

        assert!(
            name.contains("dataset-中文-01"),
            "archive filename should preserve unicode dataset names"
        );
    }
}
