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
//! Events are published only after the primary state change (database +
//! filesystem) has succeeded. Consumers must not assume that receiving an
//! event means no later rollback occurred at a higher level — but in
//! practice the service layer guarantees this ordering.

use crate::dataset::model::DatasetRecord;

/// A dataset lifecycle event carrying the resulting record state.
#[derive(Clone, Debug)]
pub enum DatasetEvent {
    /// A new dataset was created (ingest or import).
    Created(DatasetRecord),
    /// An existing dataset was modified (metadata update, trash, restore,
    /// delete, re-import).
    Updated(DatasetRecord),
}

/// Port for publishing dataset events.
///
/// Implementations are provided by the application/transport layer. The
/// trait is deliberately synchronous and infallible: event delivery is
/// best-effort and must not block or fail the primary workflow.
pub(crate) trait DatasetEventPublisher {
    fn publish(&self, event: DatasetEvent);
}
