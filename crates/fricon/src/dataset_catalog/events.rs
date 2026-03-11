use crate::{
    dataset_catalog::DatasetRecord,
    runtime::app::{AppEvent, AppState},
};

pub(super) fn emit_dataset_updated(state: &AppState, record: DatasetRecord) {
    let DatasetRecord { id, metadata } = record;
    let crate::dataset_catalog::DatasetMetadata {
        name,
        description,
        favorite,
        tags,
        status,
        created_at,
        ..
    } = metadata;

    let _ = state
        .event_sender
        .send(AppEvent::DatasetUpdated {
            id,
            name,
            description,
            favorite,
            tags,
            status,
            created_at,
        });
}
