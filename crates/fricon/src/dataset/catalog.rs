//! Dataset catalog module — defines the repository port and re-exports the
//! catalog service and error types.
//!
//! The [`DatasetCatalogRepository`] trait is the persistence boundary for all
//! catalog operations. Implementations live in `database::dataset`.

mod error;
mod service;

pub use self::error::CatalogError;
pub(crate) use self::service::DatasetCatalogService;
use crate::dataset::{
    NormalizedTag,
    model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
    portability::ExportedMetadata,
};

/// Persistence port for dataset catalog operations.
///
/// # Ownership
///
/// Implementations own all SQL / persistence details. The service layer
/// calls these methods inside its orchestration workflows and is responsible
/// for coordinating filesystem and event side effects around them.
///
/// # Invariants
///
/// - Tag mutation methods (`add_tags`, `remove_tags`, `delete_tag`,
///   `rename_tag`, `merge_tag`) must be transactional: either all tag changes
///   within one call succeed or none do.
/// - `mark_dataset_deleted` must set `deleted_at` and return the updated record
///   atomically.
/// - Import methods (`insert_imported_dataset_record`,
///   `replace_imported_dataset_record`) must preserve the archive's `uid`,
///   `created_at`, `favorite`, and `status` fields exactly.
///
/// # Extension notes
///
/// Adding a new metadata field requires updating both import methods, the
/// adapter in `database::dataset`, and [`ExportedMetadata`].
#[cfg_attr(test, mockall::automock)]
pub(crate) trait DatasetCatalogRepository: Send + Sync {
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError>;
    fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError>;
    /// List all datasets including those marked as permanently deleted.
    ///
    /// Used by reconciliation and garbage collection workflows to inspect
    /// the full dataset inventory.
    fn list_all_datasets_including_deleted(&self) -> Result<Vec<DatasetRecord>, CatalogError>;
    /// List only datasets that have been permanently deleted (`deleted_at`
    /// set). Used by garbage collection to find graveyard candidates.
    fn list_deleted_datasets(&self) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError>;
    fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), CatalogError>;
    fn add_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError>;
    fn remove_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError>;
    /// Set `deleted_at` on the record and return the tombstoned record.
    ///
    /// Must be atomic: the returned record reflects the `deleted_at` value
    /// that was just written.
    fn mark_dataset_deleted(&self, id: i32) -> Result<DatasetRecord, CatalogError>;
    /// Set `trashed_at` to the current timestamp. Does not touch the live
    /// filesystem — the service layer decides when to move data.
    fn trash_dataset(&self, id: i32) -> Result<(), CatalogError>;
    /// Clear `trashed_at`, making the dataset live again.
    fn restore_dataset(&self, id: i32) -> Result<(), CatalogError>;
    /// Delete a tag globally. Must run in a transaction to remove both the
    /// tag row and all association rows.
    fn delete_tag(&self, tag: &NormalizedTag) -> Result<(), CatalogError>;
    /// Rename a tag. Must be transactional.
    fn rename_tag(
        &self,
        old_name: &NormalizedTag,
        new_name: &NormalizedTag,
    ) -> Result<(), CatalogError>;
    /// Merge `source` tag into `target`: reassign all associations from
    /// `source` to `target` and delete `source`. Must be transactional.
    fn merge_tag(&self, source: &NormalizedTag, target: &NormalizedTag)
    -> Result<(), CatalogError>;

    /// Insert a new dataset record using the uid and metadata from an imported
    /// archive.
    ///
    /// Implementations create any missing tags from `metadata.tags` and must
    /// preserve the imported `created_at`, `favorite`, and `status` fields.
    fn insert_imported_dataset_record(
        &self,
        metadata: &ExportedMetadata,
    ) -> Result<DatasetRecord, CatalogError>;

    /// Replace an existing dataset record in place using metadata from an
    /// imported archive.
    ///
    /// The existing record id must be preserved. Any trashed/deleted state is
    /// cleared so the imported dataset becomes live again, and tag
    /// associations are rebuilt from `metadata.tags`.
    fn replace_imported_dataset_record(
        &self,
        id: i32,
        metadata: &ExportedMetadata,
    ) -> Result<DatasetRecord, CatalogError>;

    /// Find a dataset record by its uuid without requiring the DB id.
    fn find_dataset_by_uid(&self, uid: uuid::Uuid) -> Result<Option<DatasetRecord>, CatalogError>;
}
