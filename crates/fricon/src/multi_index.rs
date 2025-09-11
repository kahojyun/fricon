#![allow(clippy::float_cmp, clippy::match_same_arms, clippy::doc_markdown)]
//! Multi-index inference utilities
//!
//! Fast heuristic: compare only the first two logical rows across batches and
//! pick the first column (by order) whose values differ. The inferred multi-index
//! is all columns up to and including that column; that column is the deepest level.

use arrow::array::{
    Array, BooleanArray, LargeStringArray, PrimitiveArray, RecordBatch, StringArray,
};
use arrow::datatypes::{DataType, SchemaRef};
use serde::{Deserialize, Serialize};

/// Result of multi-index inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiIndex {
    /// Column indices (0-based) that compose the multi-index, ordered from
    /// highest to deepest level. Empty if no multi-index inferred.
    pub level_indices: Vec<usize>,
    /// Column names corresponding to `level_indices`.
    pub level_names: Vec<String>,
    /// The index of the deepest level (last) in the original schema columns,
    /// if any.
    pub deepest_level_col: Option<usize>,
}

impl MultiIndex {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.level_indices.is_empty()
    }
}

/// Infer multi-index levels from a sequence of batches (same schema).
///
/// Contract (simplified):
/// - Compare only row 0 and row 1 across the concatenated batches.
/// - Pick the first column (by order) whose values differ between those two rows.
/// - Levels are all columns 0..=that_index; otherwise empty if fewer than 2 rows
///   or no difference found.
/// - Null handling: null vs non-null counts as different; two nulls are equal.
#[must_use]
pub fn infer_multi_index_from_batches(batches: &[RecordBatch]) -> MultiIndex {
    let schema: Option<SchemaRef> = batches.first().map(RecordBatch::schema);
    let Some(schema) = schema else {
        return MultiIndex {
            level_indices: Vec::new(),
            level_names: Vec::new(),
            deepest_level_col: None,
        };
    };
    let ncols = schema.fields().len();
    if ncols == 0 {
        return MultiIndex {
            level_indices: Vec::new(),
            level_names: Vec::new(),
            deepest_level_col: None,
        };
    }

    // Locate first two logical row positions across batches
    let Some([r0, r1]) = first_two_positions(batches) else {
        return MultiIndex {
            level_indices: Vec::new(),
            level_names: Vec::new(),
            deepest_level_col: None,
        };
    };

    // Find the first differing column between row0 and row1
    let mut deepest: Option<usize> = None;
    for col in 0..ncols {
        if column_differs(batches, col, r0, r1) {
            deepest = Some(col);
            break;
        }
    }

    let Some(deepest_idx) = deepest else {
        return MultiIndex {
            level_indices: Vec::new(),
            level_names: Vec::new(),
            deepest_level_col: None,
        };
    };

    let level_indices: Vec<usize> = (0..=deepest_idx).collect();
    let level_names: Vec<String> = level_indices
        .iter()
        .map(|&i| schema.field(i).name().to_string())
        .collect();

    MultiIndex {
        level_indices,
        level_names,
        deepest_level_col: Some(deepest_idx),
    }
}

fn first_two_positions(batches: &[RecordBatch]) -> Option<[(usize, usize); 2]> {
    let mut it = batches.iter().enumerate().filter(|(_, b)| b.num_rows() > 0);
    let (b0_idx, b0) = it.next()?;
    let r0 = (b0_idx, 0);
    let r1 = if b0.num_rows() >= 2 {
        (b0_idx, 1)
    } else {
        let (b1_idx, _b1) = it.next()?;
        (b1_idx, 0)
    };
    Some([r0, r1])
}

fn column_differs(
    batches: &[RecordBatch],
    col: usize,
    r0: (usize, usize),
    r1: (usize, usize),
) -> bool {
    let dt = batches[r0.0].schema().field(col).data_type().clone();
    #[allow(clippy::unnested_or_patterns)]
    match dt {
        DataType::Boolean => diff_bool(batches, col, r0, r1),
        DataType::Utf8 => diff_string32(batches, col, r0, r1),
        DataType::LargeUtf8 => diff_string64(batches, col, r0, r1),
        DataType::Int8 => diff_prim::<arrow::datatypes::Int8Type>(batches, col, r0, r1),
        DataType::Int16 => diff_prim::<arrow::datatypes::Int16Type>(batches, col, r0, r1),
        DataType::Int32 => diff_prim::<arrow::datatypes::Int32Type>(batches, col, r0, r1),
        DataType::Int64 => diff_prim::<arrow::datatypes::Int64Type>(batches, col, r0, r1),
        DataType::UInt8 => diff_prim::<arrow::datatypes::UInt8Type>(batches, col, r0, r1),
        DataType::UInt16 => diff_prim::<arrow::datatypes::UInt16Type>(batches, col, r0, r1),
        DataType::UInt32 => diff_prim::<arrow::datatypes::UInt32Type>(batches, col, r0, r1),
        DataType::UInt64 => diff_prim::<arrow::datatypes::UInt64Type>(batches, col, r0, r1),
        DataType::Float16 => diff_prim::<arrow::datatypes::Float16Type>(batches, col, r0, r1),
        DataType::Float32 => diff_prim::<arrow::datatypes::Float32Type>(batches, col, r0, r1),
        DataType::Float64 => diff_prim::<arrow::datatypes::Float64Type>(batches, col, r0, r1),
        DataType::Date32 => diff_prim::<arrow::datatypes::Date32Type>(batches, col, r0, r1),
        DataType::Date64 => diff_prim::<arrow::datatypes::Date64Type>(batches, col, r0, r1),
        DataType::Timestamp(unit, tz) => match (unit, tz) {
            (arrow::datatypes::TimeUnit::Second, None)
            | (arrow::datatypes::TimeUnit::Second, Some(_)) => {
                diff_prim::<arrow::datatypes::TimestampSecondType>(batches, col, r0, r1)
            }
            (arrow::datatypes::TimeUnit::Millisecond, None)
            | (arrow::datatypes::TimeUnit::Millisecond, Some(_)) => {
                diff_prim::<arrow::datatypes::TimestampMillisecondType>(batches, col, r0, r1)
            }
            (arrow::datatypes::TimeUnit::Microsecond, None)
            | (arrow::datatypes::TimeUnit::Microsecond, Some(_)) => {
                diff_prim::<arrow::datatypes::TimestampMicrosecondType>(batches, col, r0, r1)
            }
            (arrow::datatypes::TimeUnit::Nanosecond, None)
            | (arrow::datatypes::TimeUnit::Nanosecond, Some(_)) => {
                diff_prim::<arrow::datatypes::TimestampNanosecondType>(batches, col, r0, r1)
            }
        },
        DataType::Time32(unit) => match unit {
            arrow::datatypes::TimeUnit::Second => {
                diff_prim::<arrow::datatypes::Time32SecondType>(batches, col, r0, r1)
            }
            arrow::datatypes::TimeUnit::Millisecond => {
                diff_prim::<arrow::datatypes::Time32MillisecondType>(batches, col, r0, r1)
            }
            _ => false,
        },
        DataType::Time64(unit) => match unit {
            arrow::datatypes::TimeUnit::Microsecond => {
                diff_prim::<arrow::datatypes::Time64MicrosecondType>(batches, col, r0, r1)
            }
            arrow::datatypes::TimeUnit::Nanosecond => {
                diff_prim::<arrow::datatypes::Time64NanosecondType>(batches, col, r0, r1)
            }
            _ => false,
        },
        _ => false,
    }
}

fn diff_bool(batches: &[RecordBatch], col: usize, r0: (usize, usize), r1: (usize, usize)) -> bool {
    let a0 = batches[r0.0].column(col);
    let a1 = batches[r1.0].column(col);
    let (Some(b0), Some(b1)) = (
        a0.as_any().downcast_ref::<BooleanArray>(),
        a1.as_any().downcast_ref::<BooleanArray>(),
    ) else {
        return false;
    };
    let v0 = if b0.is_valid(r0.1) {
        Some(b0.value(r0.1))
    } else {
        None
    };
    let v1 = if b1.is_valid(r1.1) {
        Some(b1.value(r1.1))
    } else {
        None
    };
    v0 != v1
}

fn diff_string32(
    batches: &[RecordBatch],
    col: usize,
    r0: (usize, usize),
    r1: (usize, usize),
) -> bool {
    let a0 = batches[r0.0].column(col);
    let a1 = batches[r1.0].column(col);
    let (Some(s0), Some(s1)) = (
        a0.as_any().downcast_ref::<StringArray>(),
        a1.as_any().downcast_ref::<StringArray>(),
    ) else {
        return false;
    };
    let v0 = if s0.is_valid(r0.1) {
        Some(s0.value(r0.1))
    } else {
        None
    };
    let v1 = if s1.is_valid(r1.1) {
        Some(s1.value(r1.1))
    } else {
        None
    };
    v0 != v1
}

fn diff_string64(
    batches: &[RecordBatch],
    col: usize,
    r0: (usize, usize),
    r1: (usize, usize),
) -> bool {
    let a0 = batches[r0.0].column(col);
    let a1 = batches[r1.0].column(col);
    let (Some(s0), Some(s1)) = (
        a0.as_any().downcast_ref::<LargeStringArray>(),
        a1.as_any().downcast_ref::<LargeStringArray>(),
    ) else {
        return false;
    };
    let v0 = if s0.is_valid(r0.1) {
        Some(s0.value(r0.1))
    } else {
        None
    };
    let v1 = if s1.is_valid(r1.1) {
        Some(s1.value(r1.1))
    } else {
        None
    };
    v0 != v1
}

fn diff_prim<T: arrow::datatypes::ArrowPrimitiveType>(
    batches: &[RecordBatch],
    col: usize,
    r0: (usize, usize),
    r1: (usize, usize),
) -> bool {
    let a0 = batches[r0.0].column(col);
    let a1 = batches[r1.0].column(col);
    let (Some(p0), Some(p1)) = (
        a0.as_any().downcast_ref::<PrimitiveArray<T>>(),
        a1.as_any().downcast_ref::<PrimitiveArray<T>>(),
    ) else {
        return false;
    };
    let v0 = if p0.is_valid(r0.1) {
        Some(p0.value(r0.1))
    } else {
        None
    };
    let v1 = if p1.is_valid(r1.1) {
        Some(p1.value(r1.1))
    } else {
        None
    };
    v0 != v1
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int32Array;
    use arrow::datatypes::{Field, Schema};
    use std::sync::Arc;

    fn make_batch(a_vals: &[i32], b_vals: &[&str], c_vals: &[i32]) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("A", DataType::Int32, false),
            Field::new("B", DataType::Utf8, false),
            Field::new("C", DataType::Int32, false),
        ]));
        let a = Int32Array::from(a_vals.to_vec());
        let b = StringArray::from(b_vals.to_vec());
        let c = Int32Array::from(c_vals.to_vec());
        RecordBatch::try_new(schema, vec![Arc::new(a), Arc::new(b), Arc::new(c)]).unwrap()
    }

    #[test]
    fn infer_by_first_two_rows_first_diff_column() {
        // Row0 vs Row1: A=0 vs 0 (same), B=x vs y (first diff), C=10 vs 11 (also diff)
        // Heuristic picks the first differing column by order => B (index 1)
        let b1 = make_batch(
            &[0, 0, 0, 1, 1],
            &["x", "y", "z", "x", "y"],
            &[10, 11, 12, 13, 14],
        );
        let b2 = make_batch(
            &[1, 1, 2, 2, 2],
            &["z", "x", "x", "y", "z"],
            &[15, 16, 17, 18, 19],
        );
        let mi = infer_multi_index_from_batches(&[b1, b2]);
        assert_eq!(mi.deepest_level_col, Some(1));
        assert_eq!(mi.level_indices, vec![0, 1]);
        assert_eq!(mi.level_names, vec!["A", "B"]);
    }

    #[test]
    fn infer_empty_when_no_full_change() {
        // No column changes every row (A and B repeat; C repeats as well)
        let b = make_batch(&[0, 0, 1, 1], &["x", "x", "y", "y"], &[5, 5, 6, 6]);
        let mi = infer_multi_index_from_batches(&[b]);
        assert!(mi.is_empty());
        assert_eq!(mi.deepest_level_col, None);
    }

    #[test]
    fn first_column_is_deepest_if_it_differs_in_first_two_rows() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("A", DataType::Int32, false),
            Field::new("B", DataType::Utf8, true),
        ]));
        let a = Int32Array::from(vec![1, 2, 3, 4]);
        let b = StringArray::from(vec![Some("x"), None, Some("x"), None]);
        let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(a), Arc::new(b)]).unwrap();
        let mi = infer_multi_index_from_batches(&[batch]);
        assert_eq!(mi.deepest_level_col, Some(0));
        assert_eq!(mi.level_indices, vec![0]);
        assert_eq!(mi.level_names, vec!["A"]);
    }

    #[test]
    fn nulls_equal_and_diff_against_non_null_in_first_two_rows() {
        // First two rows: A=Some(1) vs Some(2) => diff on column 0
        let schema = Arc::new(Schema::new(vec![
            Field::new("A", DataType::Int32, true),
            Field::new("C", DataType::Int32, true),
        ]));
        let a = Int32Array::from(vec![Some(1), Some(2), Some(3), Some(4)]);
        let c = Int32Array::from(vec![Some(0), None, None, Some(1)]);
        let b = RecordBatch::try_new(schema, vec![Arc::new(a), Arc::new(c)]).unwrap();
        let mi = infer_multi_index_from_batches(&[b]);
        assert_eq!(mi.deepest_level_col, Some(0)); // A differs between first two rows
    }
}
