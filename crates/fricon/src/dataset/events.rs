use crate::dataset::model::DatasetRecord;

#[derive(Clone, Debug)]
pub enum DatasetEvent {
    Created(DatasetRecord),
    Updated(DatasetRecord),
}

pub(crate) trait DatasetEventPublisher {
    fn publish(&self, event: DatasetEvent);
}
