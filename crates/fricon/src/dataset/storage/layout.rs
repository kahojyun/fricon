use std::path::{Path, PathBuf};

pub(crate) const MANIFEST_FILENAME: &str = "dataset_manifest.json";

/// Generate a chunk filename for the given chunk index.
pub(crate) fn chunk_filename(chunk_index: usize) -> String {
    format!("data_chunk_{chunk_index}.arrow")
}

/// Get the chunk path by joining the base path with the chunk filename.
pub(crate) fn chunk_path(dir_path: &Path, chunk_index: usize) -> PathBuf {
    dir_path.join(chunk_filename(chunk_index))
}

pub(crate) fn manifest_path(dir_path: &Path) -> PathBuf {
    dir_path.join(MANIFEST_FILENAME)
}
