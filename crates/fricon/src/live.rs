use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use arrow::{array::RecordBatch, datatypes::SchemaRef};
use tokio::sync::broadcast;

/// Live, in-memory representation of a dataset while it is being written.
///
/// Provides:
/// - Append semantics mirroring what's persisted
/// - Lightweight event stream for UI/reactive consumers
/// - Row selection (by sorted unique indices) with optional column projection
/// - Ability to replace a sequential front segment (used by future compaction /
///   reordering)
#[derive(Debug, Clone)]
pub struct LiveDatasetWriter {
    inner: Arc<LiveInner>,
}

/// Shareable read-only (logical) view of a live in-memory dataset.
///
/// Cloning is cheap (Arc clone). Mutation APIs are exposed only via
/// `LiveDatasetWriter` ensuring clearer ownership around who may append.
#[derive(Debug, Clone)]
pub struct LiveDataset {
    inner: Arc<LiveInner>,
}

#[derive(Debug)]
struct LiveInner {
    schema: SchemaRef,
    batches: RwLock<VecDeque<RecordBatch>>, // ordered batches
    total_rows: RwLock<usize>,              // cached total rows
    event_tx: broadcast::Sender<LiveEvent>,
    replaced_front_rows: RwLock<usize>, // cursor advanced by replace_sequential_front
}

#[derive(Debug, Clone)]
pub enum LiveEvent {
    Appended { new_rows: usize, total_rows: usize },
    SequentialFrontReplaced { replaced_rows: usize, cursor: usize },
    Closed,
}

impl LiveDatasetWriter {
    /// Create a new writer (and internal shared state). Use `live()` to obtain
    /// a `LiveDataset` reader handle. Prefer this over the older `new_pair`.
    pub fn new(schema: SchemaRef) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        let inner = Arc::new(LiveInner {
            schema,
            batches: RwLock::new(VecDeque::new()),
            total_rows: RwLock::new(0),
            event_tx,
            replaced_front_rows: RwLock::new(0),
        });
        Self { inner }
    }

    pub fn append(&self, batch: RecordBatch) {
        if batch.schema() != self.inner.schema {
            return;
        }
        let rows = batch.num_rows();
        let mut total = self
            .inner
            .total_rows
            .write()
            .expect("Lock should not be poisoned as the critical section doesn't panic");
        let mut vec = self
            .inner
            .batches
            .write()
            .expect("Lock should not be poisoned as the critical section doesn't panic");
        vec.push_back(batch);
        *total += rows;
        let _ = self.inner.event_tx.send(LiveEvent::Appended {
            new_rows: rows,
            total_rows: *total,
        });
    }
    pub fn replace_sequential_front(
        &self,
        new_batches: &[RecordBatch],
    ) -> Result<(), ReplaceError> {
        if new_batches.is_empty() {
            return Ok(());
        }
        for b in new_batches {
            if b.schema() != self.inner.schema {
                return Err(ReplaceError::SchemaMismatch);
            }
        }
        let new_rows: usize = new_batches.iter().map(RecordBatch::num_rows).sum();
        if new_rows == 0 {
            return Ok(());
        }
        let mut batches = self
            .inner
            .batches
            .write()
            .map_err(|_| ReplaceError::Poison)?;
        let mut cursor = self
            .inner
            .replaced_front_rows
            .write()
            .map_err(|_| ReplaceError::Poison)?;
        let total = *self
            .inner
            .total_rows
            .read()
            .map_err(|_| ReplaceError::Poison)?;
        if *cursor > total {
            return Err(ReplaceError::OutOfBounds);
        }
        if total - *cursor < new_rows {
            return Err(ReplaceError::InsufficientRows);
        }
        let mut remaining = new_rows;
        while remaining > 0 {
            let Some(front) = batches.pop_front() else {
                return Err(ReplaceError::InsufficientRows);
            };
            let rows = front.num_rows();
            if rows <= remaining {
                remaining -= rows;
            } else {
                let tail = front.slice(remaining, rows - remaining);
                batches.push_front(tail);
                remaining = 0;
            }
        }
        for b in new_batches.iter().rev() {
            batches.push_front(b.clone());
        }
        *cursor += new_rows;
        let cursor_val = *cursor;
        drop(cursor);
        drop(batches);
        let _ = self
            .inner
            .event_tx
            .send(LiveEvent::SequentialFrontReplaced {
                replaced_rows: new_rows,
                cursor: cursor_val,
            });
        Ok(())
    }
    /// Obtain a shareable read-only handle.
    pub fn reader(&self) -> LiveDataset {
        LiveDataset {
            inner: self.inner.clone(),
        }
    }
}

impl LiveDataset {
    pub fn schema(&self) -> SchemaRef {
        self.inner.schema.clone()
    }
    pub fn subscribe(&self) -> broadcast::Receiver<LiveEvent> {
        self.inner.event_tx.subscribe()
    }
    pub fn total_rows(&self) -> usize {
        *self
            .inner
            .total_rows
            .read()
            .expect("Read lock should not be poisoned as the critical section doesn't panic")
    }
    pub fn tail(&self, n: usize) -> Option<RecordBatch> {
        use arrow::compute::concat_batches;
        let vec = self
            .inner
            .batches
            .read()
            .expect("Read lock should not be poisoned as the critical section doesn't panic");
        if vec.is_empty() || n == 0 {
            return None;
        }
        let mut collected = Vec::new();
        let mut rows = 0usize;
        for batch in vec.iter().rev() {
            collected.push(batch.clone());
            rows += batch.num_rows();
            if rows >= n {
                break;
            }
        }
        collected.reverse();
        let schema_ref = self.inner.schema.clone();
        concat_batches(&schema_ref, &collected).ok()
    }

    pub fn select_by_indices(
        &self,
        indices: &[usize],
        column_indices: Option<&[usize]>,
    ) -> Result<RecordBatch, SelectError> {
        let batches = self.inner.batches.read().map_err(|_| SelectError::Poison)?;
        if batches.is_empty() {
            return self.empty_selection(indices, column_indices);
        }
        Self::validate_indices_sorted_unique(indices)?;
        Self::validate_indices_in_bounds(indices, &batches)?;
        if indices.is_empty() {
            return self.empty_selection(indices, column_indices);
        }
        let cols = self.resolve_columns(column_indices)?;
        let mapping = Self::build_row_mapping(indices, &batches);
        self.interleave_projected(&batches, &mapping, &cols)
    }

    fn empty_selection(
        &self,
        indices: &[usize],
        column_indices: Option<&[usize]>,
    ) -> Result<RecordBatch, SelectError> {
        use arrow::array::new_empty_array;
        if !indices.is_empty() {
            return Err(SelectError::OutOfBounds);
        }
        let base = self.inner.schema.clone();
        let cols = self.resolve_columns(column_indices)?;
        let projected_schema = if cols.len() == base.fields().len() {
            base
        } else {
            let projected = base
                .as_ref()
                .project(&cols)
                .map_err(|e| SelectError::Arrow(e.to_string()))?;
            Arc::new(projected)
        };
        let arrays = projected_schema
            .fields()
            .iter()
            .map(|f| new_empty_array(f.data_type()))
            .collect();
        RecordBatch::try_new(projected_schema, arrays)
            .map_err(|e| SelectError::Arrow(e.to_string()))
    }

    fn validate_indices_sorted_unique(indices: &[usize]) -> Result<(), SelectError> {
        if indices.windows(2).any(|w| w[0] >= w[1]) {
            return Err(SelectError::NotSortedUnique);
        }
        Ok(())
    }
    fn validate_indices_in_bounds(
        indices: &[usize],
        batches: &VecDeque<RecordBatch>,
    ) -> Result<(), SelectError> {
        if let Some(&last) = indices.last() {
            let total: usize = batches.iter().map(RecordBatch::num_rows).sum();
            if last >= total {
                return Err(SelectError::OutOfBounds);
            }
        }
        Ok(())
    }
    fn resolve_columns(&self, column_indices: Option<&[usize]>) -> Result<Vec<usize>, SelectError> {
        let schema = self.inner.schema.clone();
        let max = schema.fields().len();
        let cols = match column_indices {
            None => (0..max).collect(),
            Some(list) => {
                for &c in list {
                    if c >= max {
                        return Err(SelectError::ColumnOutOfRange);
                    }
                }
                list.to_vec()
            }
        };
        Ok(cols)
    }
    fn build_row_mapping(
        indices: &[usize],
        batches: &VecDeque<RecordBatch>,
    ) -> Vec<(usize, usize)> {
        let mut mapping = Vec::with_capacity(indices.len());
        let mut prefix = 0usize;
        for (bi, b) in batches.iter().enumerate() {
            let next_prefix = prefix + b.num_rows();
            let start_pos = match indices.binary_search(&prefix) {
                Ok(pos) | Err(pos) => pos,
            };
            for &g in &indices[start_pos..] {
                if g < prefix {
                    continue;
                }
                if g >= next_prefix {
                    break;
                }
                mapping.push((bi, g - prefix));
            }
            prefix = next_prefix;
            if mapping.len() == indices.len() {
                break;
            }
        }
        mapping
    }
    fn interleave_projected(
        &self,
        batches: &VecDeque<RecordBatch>,
        mapping: &[(usize, usize)],
        cols: &[usize],
    ) -> Result<RecordBatch, SelectError> {
        use arrow::compute::interleave_record_batch;
        if mapping.is_empty() {
            return self.empty_selection(&[], Some(cols));
        }
        let full_width = self.inner.schema.fields().len();
        let mut used = vec![false; batches.len()];
        for (bi, _) in mapping {
            used[*bi] = true;
        }
        let used_count = used.iter().filter(|u| **u).count();
        let mut projected = Vec::with_capacity(used_count);
        for (i, b) in batches.iter().enumerate() {
            if used[i] {
                projected.push(if cols.len() == full_width {
                    b.clone()
                } else {
                    b.project(cols)
                        .map_err(|e| SelectError::Arrow(e.to_string()))?
                });
            }
        }
        let mut remap = vec![usize::MAX; batches.len()];
        let mut next = 0;
        for (i, u) in used.iter().enumerate() {
            if *u {
                remap[i] = next;
                next += 1;
            }
        }
        let compressed: Vec<(usize, usize)> =
            mapping.iter().map(|(bi, lr)| (remap[*bi], *lr)).collect();
        let refs: Vec<&RecordBatch> = projected.iter().collect();
        interleave_record_batch(&refs, &compressed).map_err(|e| SelectError::Arrow(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReplaceError {
    #[error("range out of bounds")]
    OutOfBounds,
    #[error("schema mismatch")]
    SchemaMismatch,
    #[error("lock poisoned")]
    Poison,
    #[error("insufficient rows to replace")]
    InsufficientRows,
}

#[derive(Debug, thiserror::Error)]
pub enum SelectError {
    #[error("out of bounds index")]
    OutOfBounds,
    #[error("column out of range")]
    ColumnOutOfRange,
    #[error("index too large")]
    IndexTooLarge,
    #[error("poisoned lock")]
    Poison,
    #[error("arrow error: {0}")]
    Arrow(String),
    #[error("indices not strictly increasing unique")]
    NotSortedUnique,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::{
        array::Int32Array,
        datatypes::{DataType, Field, Schema},
    };

    use super::*;
    fn make_schema() -> SchemaRef {
        Arc::new(Schema::new(vec![Field::new("v", DataType::Int32, false)]))
    }
    fn make_batch(schema: &SchemaRef, start: i32, n: i32) -> RecordBatch {
        let arr = Int32Array::from_iter_values(start..start + n);
        RecordBatch::try_new(schema.clone(), vec![Arc::new(arr)]).unwrap()
    }
    #[test]
    fn live_dataset_append_and_tail() {
        let schema = make_schema();
        let writer = LiveDatasetWriter::new(schema.clone());
        let live = writer.reader();
        for i in 0..5 {
            writer.append(make_batch(&schema, i * 10, 10));
        }
        assert_eq!(live.total_rows(), 50);
        let tail = live.tail(15).unwrap();
        assert_eq!(tail.num_rows(), 20);
    }
    #[test]
    fn events_fire() {
        let schema = make_schema();
        let writer = LiveDatasetWriter::new(schema.clone());
        let live = writer.reader();
        let mut rx = live.subscribe();
        writer.append(make_batch(&schema, 0, 5));
        let evt = rx.try_recv().unwrap();
        match evt {
            LiveEvent::Appended {
                new_rows,
                total_rows,
            } => {
                assert_eq!(new_rows, 5);
                assert_eq!(total_rows, 5);
            }
            _ => panic!("unexpected"),
        }
    }
    #[test]
    fn sequential_front_replace_partial_end() {
        let schema = make_schema();
        let writer = LiveDatasetWriter::new(schema.clone());
        let live = writer.reader();
        for i in 0..3 {
            writer.append(make_batch(&schema, i * 10, 10));
        }
        assert_eq!(live.total_rows(), 30);
        let nb1 = make_batch(&schema, 1000, 8);
        let nb2 = make_batch(&schema, 2000, 7);
        writer.replace_sequential_front(&[nb1, nb2]).unwrap();
        assert_eq!(live.total_rows(), 30);
        let all = live.tail(30).unwrap();
        let col = all.column(0).as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(col.value(0), 1000);
        assert_eq!(col.value(7), 1000 + 7);
        assert_eq!(col.value(8), 2000);
    }
    #[test]
    fn select_by_indices_basic() {
        let schema = make_schema();
        let writer = LiveDatasetWriter::new(schema.clone());
        let live = writer.reader();
        for i in 0..3 {
            writer.append(make_batch(&schema, i * 10, 10));
        }
        let indices = [0usize, 5, 12, 29];
        let batch = live.select_by_indices(&indices, None).unwrap();
        assert_eq!(batch.num_rows(), indices.len());
        let col = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(col.value(0), 0);
        assert_eq!(col.value(1), 5);
        assert_eq!(col.value(2), 12);
        assert_eq!(col.value(3), 29);
    }
    #[test]
    fn select_by_indices_empty_and_oob() {
        let schema = make_schema();
        let writer = LiveDatasetWriter::new(schema.clone());
        let live = writer.reader();
        let empty = live.select_by_indices(&[], None).unwrap();
        assert_eq!(empty.num_rows(), 0);
        assert!(matches!(
            live.select_by_indices(&[0], None),
            Err(SelectError::OutOfBounds)
        ));
    }
    #[test]
    fn select_by_indices_multi_column_subset() {
        use arrow::{array::Int32Array, datatypes::Schema};
        let schema: SchemaRef = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
            Field::new("c", DataType::Int32, false),
        ]));
        let writer = LiveDatasetWriter::new(schema.clone());
        let live = writer.reader();
        let make_tri_batch = |start: i32, n: i32| {
            let a = Int32Array::from_iter_values(start..start + n);
            let b = Int32Array::from_iter_values((start * 10)..(start * 10 + n));
            let c = Int32Array::from_iter_values((start * 100)..(start * 100 + n));
            RecordBatch::try_new(schema.clone(), vec![Arc::new(a), Arc::new(b), Arc::new(c)])
                .unwrap()
        };
        writer.append(make_tri_batch(0, 5));
        writer.append(make_tri_batch(5, 5));
        let indices = [1usize, 7];
        let batch = live.select_by_indices(&indices, Some(&[1, 2])).unwrap();
        assert_eq!(batch.num_columns(), 2);
        let bcol = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        let ccol = batch
            .column(1)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(bcol.value(0), 1);
        assert_eq!(bcol.value(1), 52);
        assert_eq!(ccol.value(0), 1);
        assert_eq!(ccol.value(1), 502);
    }
}
