use std::{
    borrow::Cow,
    collections::{Bound, VecDeque},
    ops::{Range, RangeBounds},
};

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;

use crate::dataset::Error;

/// Manage chunked batches. Able to release batches from front to a target row,
/// while ensuring `target_row..` can be accessed.
#[derive(Debug)]
pub struct ChunkedTable {
    schema: SchemaRef,
    batches: VecDeque<RecordBatch>,
    offsets: VecDeque<usize>,
}

impl ChunkedTable {
    pub fn new(schema: SchemaRef) -> Self {
        Self {
            schema,
            batches: VecDeque::new(),
            offsets: VecDeque::from([0]),
        }
    }

    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    pub fn last_offset(&self) -> usize {
        *self.offsets.back().expect("At least one offset exists.")
    }

    pub fn first_offset(&self) -> usize {
        *self.offsets.front().expect("At least one offset exists.")
    }

    pub fn push_back(&mut self, batch: RecordBatch) -> Result<(), Error> {
        if batch.schema() != self.schema {
            return Err(Error::SchemaMismatch);
        }
        if batch.num_rows() != 0 {
            self.offsets
                .push_back(self.last_offset() + batch.num_rows());
            self.batches.push_back(batch);
        }
        Ok(())
    }

    /// Release all batches fully covered by row range `..target_row`
    pub fn release_front(&mut self, target_row: usize) {
        let remove_count = self
            .offsets
            .binary_search(&target_row)
            .unwrap_or_else(|index| index.saturating_sub(1));
        self.batches.drain(..remove_count);
        self.offsets.drain(..remove_count);
    }

    pub fn range<R>(&self, range: R) -> impl Iterator<Item = Cow<'_, RecordBatch>>
    where
        R: RangeBounds<usize>,
    {
        self.range_impl(range.start_bound().cloned(), range.end_bound().cloned())
    }

    fn range_impl(
        &self,
        start: Bound<usize>,
        end: Bound<usize>,
    ) -> impl Iterator<Item = Cow<'_, RecordBatch>> {
        let start = match start {
            Bound::Included(v) => v,
            Bound::Excluded(v) => v.saturating_add(1),
            Bound::Unbounded => 0,
        }
        .max(self.first_offset());
        let end = match end {
            Bound::Included(v) => v.checked_add(1),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        }
        .unwrap_or(self.last_offset())
        .min(self.last_offset());
        let start_batch = self.offsets.binary_search(&start).unwrap_or_else(|i| i - 1);
        let end_batch = self.offsets.binary_search(&end).unwrap_or_else(|i| i);
        (start_batch..end_batch).filter_map(move |i| self.slice_batch(i, start..end))
    }

    fn slice_batch(&self, batch_index: usize, range: Range<usize>) -> Option<Cow<'_, RecordBatch>> {
        let batch = self.batches.get(batch_index)?;
        let start = self.offsets[batch_index].max(range.start);
        let end = self.offsets[batch_index + 1].min(range.end);
        (start < end).then(|| {
            if end - start < batch.num_rows() {
                Cow::Owned(batch.slice(start - self.offsets[batch_index], end - start))
            } else {
                Cow::Borrowed(batch)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{slice::SliceIndex, sync::Arc};

    use arrow_array::{ArrayRef, Int32Array, cast::AsArray, types::Int32Type};
    use arrow_select::concat::concat_batches;

    use super::*;

    fn make_batches(lengths: &[i32]) -> Vec<RecordBatch> {
        let mut start = 0;
        lengths
            .iter()
            .map(|&l| {
                let array = Int32Array::from_iter_values(start..start + l);
                start += l;
                RecordBatch::try_from_iter([("a", Arc::new(array) as ArrayRef)]).unwrap()
            })
            .collect()
    }

    #[test]
    fn chunked_table_push_back() {
        let lengths = [1, 2, 3, 4];
        let batches = make_batches(&lengths);

        let mut chunked_table = ChunkedTable::new(batches[0].schema());
        for batch in batches {
            chunked_table.push_back(batch).unwrap();
        }

        assert_eq!(chunked_table.batches.len(), lengths.len());
        assert_eq!(chunked_table.offsets, [0, 1, 3, 6, 10]);
    }

    #[test]
    fn chunked_table_release_front() {
        let lengths = [3, 3, 3, 3];
        let batches = make_batches(&lengths);
        let mut chunked_table = ChunkedTable::new(batches[0].schema());
        for batch in batches {
            chunked_table.push_back(batch).unwrap();
        }

        chunked_table.release_front(0);
        assert_eq!(chunked_table.batches.len(), 4);
        assert_eq!(chunked_table.offsets.len(), 5);

        chunked_table.release_front(1);
        assert_eq!(chunked_table.batches.len(), 4);
        assert_eq!(chunked_table.offsets.len(), 5);

        chunked_table.release_front(3);
        assert_eq!(chunked_table.batches.len(), 3);
        assert_eq!(chunked_table.offsets.len(), 4);

        chunked_table.release_front(7);
        assert_eq!(chunked_table.batches.len(), 2);
        assert_eq!(chunked_table.offsets.len(), 3);

        chunked_table.release_front(12);
        assert_eq!(chunked_table.batches.len(), 0);
        assert_eq!(chunked_table.offsets.len(), 1);

        chunked_table.release_front(15);
        assert_eq!(chunked_table.batches.len(), 0);
        assert_eq!(chunked_table.offsets.len(), 1);
    }

    fn check_slice<R>(chunked_table: &ChunkedTable, reference: &[i32], r: R)
    where
        R: RangeBounds<usize> + Clone + SliceIndex<[i32], Output = [i32]>,
    {
        let offset = chunked_table.first_offset();
        let start = r.start_bound().cloned().map(|x| x - offset);
        let end = r.end_bound().cloned().map(|x| x - offset);
        let reference = &reference[offset..];
        let batches: Vec<_> = chunked_table.range(r).collect();
        let batch =
            concat_batches(chunked_table.schema(), batches.iter().map(|x| x.as_ref())).unwrap();
        let arr = batch.column(0).as_primitive::<Int32Type>();
        assert_eq!(arr.values(), &reference[(start, end)]);
    }

    #[test]
    fn chunked_table_get_range() {
        let lengths = [3, 3, 3, 3];
        let batches = make_batches(&lengths);
        let mut chunked_table = ChunkedTable::new(batches[0].schema());
        for batch in batches {
            chunked_table.push_back(batch).unwrap();
        }
        let reference = (0..lengths.iter().sum()).collect::<Vec<_>>();

        check_slice(&chunked_table, &reference, ..);
        for i in 0..4 {
            for j in i..i + 4 {
                check_slice(&chunked_table, &reference, i..j);
                check_slice(&chunked_table, &reference, i..=j);
            }
            check_slice(&chunked_table, &reference, i..);
            check_slice(&chunked_table, &reference, ..i);
            check_slice(&chunked_table, &reference, ..=i);
        }

        chunked_table.release_front(4);

        for i in 4..8 {
            for j in i + 1..i + 4 {
                check_slice(&chunked_table, &reference, i..j);
                check_slice(&chunked_table, &reference, i..=j);
            }
            check_slice(&chunked_table, &reference, i..);
            check_slice(&chunked_table, &reference, ..i);
            check_slice(&chunked_table, &reference, ..=i);
        }
    }
}
