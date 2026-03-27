//! Dataset event types for cross-layer notifications.
//!
//! # Ownership
//!
//! This module defines the [`DatasetEvent`] enum and the
//! [`DatasetEventPublisher`] trait. The service layer publishes events;
//! the application/transport layer provides the concrete publisher
//! implementation (e.g. broadcast channel).
//!
//! # Invariants
//!
//! The catalog and ingest services publish events only after the primary
//! state change they own has succeeded. Publishers are best-effort, so
//! consumers can treat events as notifications of committed state, not as a
//! durable delivery log.

use crate::dataset::model::DatasetRecord;

/// A dataset lifecycle event carrying the resulting record state.
#[derive(Clone, Debug)]
pub enum DatasetEvent {
    /// A new dataset was created by ingest (Writing status).
    Created(DatasetRecord),
    /// A write session transitioned a dataset from Writing to Completed or
    /// Aborted.
    StatusChanged(DatasetRecord),
    /// Dataset name, description, or favorite flag changed.
    MetadataUpdated(DatasetRecord),
    /// Dataset tags were added or removed.
    TagsChanged(DatasetRecord),
    /// Dataset was moved to trash.
    Trashed(DatasetRecord),
    /// Dataset was restored from trash.
    Restored(DatasetRecord),
    /// Dataset was permanently deleted.
    Deleted(DatasetRecord),
    /// An existing dataset was replaced by a force-import.
    Imported(DatasetRecord),
}

/// Port for publishing dataset events.
///
/// Implementations are provided by the application/transport layer. The
/// trait is deliberately synchronous and infallible: event delivery is
/// best-effort and must not block or fail the primary workflow.
pub(crate) trait DatasetEventPublisher {
    fn publish(&self, event: DatasetEvent);
}

#[cfg(test)]
pub(crate) mod test_utils {
    use std::sync::Mutex;

    use super::{DatasetEvent, DatasetEventPublisher};

    /// Test double that collects all published events.
    #[derive(Default)]
    pub(crate) struct CollectEvents {
        events: Mutex<Vec<DatasetEvent>>,
    }

    impl CollectEvents {
        pub(crate) fn snapshot(&self) -> Vec<DatasetEvent> {
            self.events.lock().expect("events").clone()
        }
    }

    impl DatasetEventPublisher for CollectEvents {
        fn publish(&self, event: DatasetEvent) {
            self.events.lock().expect("events").push(event);
        }
    }
}
