use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dataset::model::{DatasetRecord, DatasetStatus};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppEvent {
    DatasetCreated {
        id: i32,
        name: String,
        description: String,
        favorite: bool,
        tags: Vec<String>,
        status: DatasetStatus,
        created_at: DateTime<Utc>,
    },
    DatasetUpdated {
        id: i32,
        name: String,
        description: String,
        favorite: bool,
        tags: Vec<String>,
        status: DatasetStatus,
        created_at: DateTime<Utc>,
    },
    ShowUiRequest,
}

#[must_use]
pub(crate) fn dataset_updated_event(record: DatasetRecord) -> AppEvent {
    let DatasetRecord { id, metadata } = record;
    AppEvent::DatasetUpdated {
        id,
        name: metadata.name,
        description: metadata.description,
        favorite: metadata.favorite,
        tags: metadata.tags,
        status: metadata.status,
        created_at: metadata.created_at,
    }
}
