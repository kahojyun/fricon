use crate::dataset::model::DatasetRecord;

#[derive(Clone, Debug)]
pub enum DatasetEvent {
    Created(DatasetRecord),
    Updated(DatasetRecord),
}

pub(crate) trait DatasetEventPublisher {
    fn publish(&self, event: DatasetEvent);
}

#[must_use]
pub(crate) fn dataset_created_event(record: DatasetRecord) -> DatasetEvent {
    DatasetEvent::Created(record)
}

#[must_use]
pub(crate) fn dataset_updated_event(record: DatasetRecord) -> DatasetEvent {
    DatasetEvent::Updated(record)
}
