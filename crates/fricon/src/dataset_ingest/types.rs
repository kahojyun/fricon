use arrow_array::RecordBatch;

#[derive(Debug, Clone)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum CreateIngestEvent {
    Batch(RecordBatch),
    Terminal(CreateTerminal),
}

#[derive(Debug, Clone)]
pub enum CreateTerminal {
    Finish,
    Abort,
}
