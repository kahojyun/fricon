use std::{
    borrow::Cow,
    fs::File,
    io,
    ops::RangeBounds,
    path::{Path, PathBuf},
    sync::Arc,
};

use arrow_array::RecordBatch;
use arrow_buffer::Buffer;
use arrow_ipc::{
    Block,
    convert::fb_to_schema,
    reader::{FileDecoder, read_footer_length},
    root_as_footer,
};
use arrow_schema::SchemaRef;
use itertools::Itertools;

use crate::{
    dataset::ChunkedTable,
    dataset_fs::{Error, chunk_path},
};

#[derive(Debug)]
pub struct ChunkReader {
    dir_path: PathBuf,
    current_chunk: usize,
    batches: Option<ChunkedTable>,
}

impl ChunkReader {
    pub fn new(dir_path: PathBuf, schema: Option<SchemaRef>) -> Self {
        Self {
            dir_path,
            current_chunk: 0,
            batches: schema.map(ChunkedTable::new),
        }
    }

    pub fn schema(&self) -> Option<&SchemaRef> {
        self.batches.as_ref().map(ChunkedTable::schema)
    }

    pub fn read_next(&mut self) -> Result<bool, Error> {
        let chunk_path = chunk_path(&self.dir_path, self.current_chunk);
        let chunk_batches = match read_ipc_file_mmap(&chunk_path) {
            Ok(batches) => batches,
            Err(Error::ChunkNotFound) => {
                return Ok(false);
            }
            Err(e) => return Err(e),
        };
        for batch in chunk_batches {
            self.batches
                .get_or_insert_with(|| ChunkedTable::new(batch.schema()))
                .push_back(batch)?;
        }
        self.current_chunk += 1;
        Ok(true)
    }

    pub fn read_all(&mut self) -> Result<(), Error> {
        while self.read_next()? {}
        Ok(())
    }

    pub fn range<R>(&self, range: R) -> impl Iterator<Item = Cow<'_, RecordBatch>>
    where
        R: RangeBounds<usize> + Copy,
    {
        self.batches.iter().flat_map(move |x| x.range(range))
    }

    pub fn num_rows(&self) -> usize {
        self.batches.as_ref().map_or(0, ChunkedTable::last_offset)
    }
}

// Based on https://github.com/apache/arrow-rs/blob/3dcd23ffa3cbc0d9496e1660c6f68ce563a336b4/arrow/examples/zero_copy_ipc.rs#L36
fn read_ipc_file_mmap(file_path: &Path) -> Result<Vec<RecordBatch>, Error> {
    let ipc_file = File::open(file_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => Error::ChunkNotFound,
        _ => Error::Io(e),
    })?;
    // SAFETY: Safe because we're only reading from the memory-mapped file and not
    // modifying it
    let mmap = unsafe { memmap2::Mmap::map(&ipc_file) }.map_err(Error::Io)?;

    // Convert the mmap region to an Arrow `Buffer`
    let bytes = bytes::Bytes::from_owner(mmap);
    let buffer = Buffer::from(bytes);

    IPCBufferDecoder::new(buffer)?.try_into_batches()
}

/// Incrementally decodes [`RecordBatch`]es from an IPC file stored in an Arrow
/// [`Buffer`] using the [`FileDecoder`] API.
struct IPCBufferDecoder {
    /// Memory (or memory mapped) Buffer with the data
    buffer: Buffer,
    /// Decoder that reads Arrays that refers to the underlying buffers
    decoder: FileDecoder,
    /// Location of the batches within the buffer
    batches: Vec<Block>,
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Casts from FlatBuffer types are safe within the context of Arrow file format"
)]
impl IPCBufferDecoder {
    fn new(buffer: Buffer) -> Result<Self, Error> {
        let (body, trailer) = buffer
            .split_last_chunk::<10>()
            .ok_or(Error::InvalidIpcFile)?;
        let footer_len = read_footer_length(trailer.to_owned())?;
        let footer = root_as_footer(
            body.get(body.len() - footer_len..)
                .ok_or(Error::InvalidIpcFile)?,
        )
        .map_err(|_| Error::InvalidIpcFile)?;

        let schema = fb_to_schema(footer.schema().ok_or(Error::InvalidIpcFile)?);

        let mut decoder = FileDecoder::new(Arc::new(schema), footer.version());

        // Read dictionaries
        for block in footer.dictionaries().iter().flatten() {
            let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
            let data = buffer.slice_with_length(block.offset() as _, block_len);
            decoder.read_dictionary(block, &data)?;
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

    fn try_into_batches(self) -> Result<Vec<RecordBatch>, Error> {
        self.batches
            .into_iter()
            .map(|block| {
                let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
                let data = self
                    .buffer
                    .slice_with_length(block.offset() as _, block_len);
                self.decoder
                    .read_record_batch(&block, &data)?
                    .ok_or(Error::InvalidIpcFile)
            })
            .try_collect()
    }
}
