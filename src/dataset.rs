use std::{fs::File, path::Path};

use arrow::{array::RecordBatchWriter, datatypes::Schema, ipc::writer::FileWriter};
use tracing::info;

pub fn create(path: &Path, schema: &Schema) -> impl RecordBatchWriter {
    let filename = path.join("dataset.arrow");
    info!("Create dataset at {:?}", filename);
    let file = File::create_new(filename).unwrap();
    FileWriter::try_new_buffered(file, schema).unwrap()
}
