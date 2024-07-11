use std::{fs::File, path::Path};

use arrow::{array::RecordBatchWriter, datatypes::Schema, ipc::writer::FileWriter};

pub fn create_dataset(path: &Path, schema: &Schema) -> impl RecordBatchWriter {
    let filename = path.join("dataset.arrow");
    let file = File::create_new(filename).unwrap();
    // FileWriter is buffered, so we don't need to wrap it in a BufWriter
    FileWriter::try_new(file, schema).unwrap()
}
