pub(crate) mod heatmap;
pub(crate) mod line;
pub(crate) mod live_line;
pub(crate) mod mapping;
pub(crate) mod scatter;

pub(crate) use self::{
    heatmap::build_heatmap_series, line::build_line_series, live_line::build_live_line_series,
    scatter::build_scatter_series,
};

#[cfg(test)]
pub(super) mod test_utils {
    use std::sync::Arc;

    use arrow_array::{Float64Array, RecordBatch};
    use arrow_schema::{DataType, Field};
    use fricon::{DatasetDataType, DatasetSchema, ScalarKind};
    use indexmap::IndexMap;

    /// Build a `RecordBatch` from named Float64 columns.
    pub(crate) fn numeric_batch(columns: &[(&str, &[f64])]) -> RecordBatch {
        let fields: Vec<_> = columns
            .iter()
            .map(|(name, _)| Field::new(*name, DataType::Float64, false))
            .collect();
        let arrays: Vec<Arc<dyn arrow_array::Array>> = columns
            .iter()
            .map(|(_, vals)| Arc::new(Float64Array::from(vals.to_vec())) as _)
            .collect();
        RecordBatch::try_new(Arc::new(arrow_schema::Schema::new(fields)), arrays).unwrap()
    }

    /// Build a `DatasetSchema` where every column is `Scalar(Numeric)`.
    pub(crate) fn numeric_schema(names: &[&str]) -> DatasetSchema {
        let columns: IndexMap<String, DatasetDataType> = names
            .iter()
            .map(|n| (n.to_string(), DatasetDataType::Scalar(ScalarKind::Numeric)))
            .collect();
        DatasetSchema::new(columns)
    }
}
