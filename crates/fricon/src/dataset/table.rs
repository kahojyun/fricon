use std::collections::VecDeque;

use arrow_arith::boolean::and;
use arrow_array::{BooleanArray, Datum, RecordBatch};
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

    /// Release all batches covered by row range `..target_row`
    pub fn release_front(&mut self, target_row: usize) {
        let remove_count = self
            .offsets
            .binary_search(&target_row)
            .unwrap_or_else(|index| index.saturating_sub(1));
        self.batches.drain(..remove_count);
        self.offsets.drain(..remove_count);
    }
}

struct Filter {
    column: usize,
    filter: Box<dyn Fn(&dyn Datum) -> BooleanArray>,
}

impl Filter {
    fn apply(&self, batch: &RecordBatch) -> BooleanArray {
        (self.filter)(batch.column(self.column))
    }
}

struct Filters(Vec<Filter>);

impl Filters {
    fn apply(&self, batch: &RecordBatch) -> Option<BooleanArray> {
        self.0
            .iter()
            .map(|x| x.apply(batch))
            .reduce(|acc, x| and(&acc, &x).expect("Should have same length."))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::RecordBatchOptions;
    use arrow_schema::Schema;

    use super::*;

    fn make_empty_batches(lengths: &[usize]) -> Vec<RecordBatch> {
        let schema = Arc::new(Schema::empty());
        lengths
            .iter()
            .map(|&l| {
                RecordBatch::try_new_with_options(
                    schema.clone(),
                    vec![],
                    &RecordBatchOptions::new().with_row_count(Some(l)),
                )
                .unwrap()
            })
            .collect()
    }

    #[test]
    fn chunked_table_push_back() {
        let lengths = [1, 2, 3, 4];
        let batches = make_empty_batches(&lengths);

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
        let batches = make_empty_batches(&lengths);
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
}
