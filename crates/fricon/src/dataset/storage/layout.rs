use std::path::{Path, PathBuf};

pub(crate) const MANIFEST_FILENAME: &str = "dataset_manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChunkKind {
    Data,
    LogicalIndex,
}

pub(crate) fn chunk_filename_for(kind: ChunkKind, chunk_index: usize) -> String {
    match kind {
        ChunkKind::Data => format!("data_chunk_{chunk_index}.arrow"),
        ChunkKind::LogicalIndex => format!("logical_index_chunk_{chunk_index}.arrow"),
    }
}

pub(crate) fn chunk_path_for(dir_path: &Path, kind: ChunkKind, chunk_index: usize) -> PathBuf {
    dir_path.join(chunk_filename_for(kind, chunk_index))
}

pub(crate) fn manifest_path(dir_path: &Path) -> PathBuf {
    dir_path.join(MANIFEST_FILENAME)
}
