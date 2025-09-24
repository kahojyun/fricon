use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use arrow::{
    array::RecordBatch,
    buffer::Buffer,
    ipc::{
        Block,
        convert::fb_to_schema,
        reader::{FileDecoder, read_footer_length},
        root_as_footer,
    },
};
use tracing::warn;

/// Generate a chunk filename for the given chunk index
pub fn chunk_filename(chunk_index: usize) -> String {
    format!("data_chunk_{chunk_index}.arrow")
}

/// Get the chunk path by joining the base path with the chunk filename
pub fn chunk_path(dir_path: &Path, chunk_index: usize) -> PathBuf {
    dir_path.join(chunk_filename(chunk_index))
}

/// Read Arrow IPC batches from a file using memory-mapped zero-copy reading
pub fn read_ipc_file_mmap(file_path: &Path) -> Result<Vec<RecordBatch>> {
    let ipc_file = File::open(file_path).context("Failed to open IPC file")?;
    let mmap = unsafe { memmap2::Mmap::map(&ipc_file) }.context("Failed to create memory map")?;

    // Convert the mmap region to an Arrow `Buffer`
    let bytes = bytes::Bytes::from_owner(mmap);
    let buffer = Buffer::from(bytes);

    // Use the IPCBufferDecoder to read batches
    let decoder = IPCBufferDecoder::new(buffer)?;

    let mut batches = Vec::new();
    for i in 0..decoder.num_batches() {
        let batch = decoder
            .get_batch(i)?
            .context("Failed to read batch from IPC file")?;
        batches.push(batch);
    }

    Ok(batches)
}

/// Incrementally decodes [`RecordBatch`]es from an IPC file stored in a Arrow
/// [`Buffer`] using the [`FileDecoder`] API.
struct IPCBufferDecoder {
    /// Memory (or memory mapped) Buffer with the data
    buffer: Buffer,
    /// Decoder that reads Arrays that refers to the underlying buffers
    decoder: FileDecoder,
    /// Location of the batches within the buffer
    batches: Vec<Block>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl IPCBufferDecoder {
    fn new(buffer: Buffer) -> Result<Self> {
        let trailer_start = buffer.len() - 10;
        let footer_len = read_footer_length(buffer[trailer_start..].try_into().unwrap())
            .context("Failed to read footer length")?;
        let footer = root_as_footer(&buffer[trailer_start - footer_len..trailer_start])
            .map_err(|e| anyhow::anyhow!("Failed to parse footer: {:?}", e))?;

        let schema = fb_to_schema(footer.schema().unwrap());

        let mut decoder = FileDecoder::new(std::sync::Arc::new(schema), footer.version());

        // Read dictionaries
        for block in footer.dictionaries().iter().flatten() {
            let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
            let data = buffer.slice_with_length(block.offset() as _, block_len);
            decoder
                .read_dictionary(block, &data)
                .context("Failed to read dictionary")?;
        }

        // convert to Vec from the flatbuffers Vector to avoid having a direct
        // dependency on flatbuffers
        let batches = footer
            .recordBatches()
            .map(|b| b.iter().copied().collect())
            .unwrap_or_default();

        Ok(Self {
            buffer,
            decoder,
            batches,
        })
    }

    /// Return the number of [`RecordBatch`]es in this buffer
    fn num_batches(&self) -> usize {
        self.batches.len()
    }

    /// Return the [`RecordBatch`] at message index `i`.
    ///
    /// This may return `None` if the IPC message was None
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn get_batch(&self, i: usize) -> Result<Option<RecordBatch>> {
        let block = &self.batches[i];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        let data = self
            .buffer
            .slice_with_length(block.offset() as _, block_len);
        self.decoder
            .read_record_batch(block, &data)
            .context("Failed to read record batch")
    }
}

#[derive(Debug)]
pub struct FileLock {
    _file: File,
    path: PathBuf,
}

impl FileLock {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .context("Failed to open file for locking.")?;
        file.try_lock().context("Failed to acquire file lock.")?;
        Ok(Self { _file: file, path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            warn!("Failed to remove locked file: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn creates_and_removes_lock_file() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");
        {
            let _lock = FileLock::new(&lock_path).expect("Should create lock");
            assert!(lock_path.exists());
        }
        // File should be removed after drop
        assert!(!lock_path.exists());
    }

    #[test]
    fn cannot_acquire_lock_twice() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("double.lock");
        let _first_lock = FileLock::new(&lock_path).expect("Should acquire first lock");
        // Attempting to acquire the same lock again should fail
        let second_lock = FileLock::new(&lock_path);
        assert!(
            second_lock.is_err(),
            "Should not acquire lock twice on same file"
        );
    }
}
