mod error;
mod service;

pub use self::error::CatalogError;
pub(crate) use self::service::DatasetCatalogService;
use crate::dataset::{
    NormalizedTag,
    model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
    portability::ExportedMetadata,
};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait DatasetCatalogRepository: Send + Sync {
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError>;
    fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn list_all_datasets_including_deleted(&self) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn list_deleted_datasets(&self) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError>;
    fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), CatalogError>;
    fn add_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError>;
    fn remove_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError>;
    fn mark_dataset_deleted(&self, id: i32) -> Result<DatasetRecord, CatalogError>;
    fn trash_dataset(&self, id: i32) -> Result<(), CatalogError>;
    fn restore_dataset(&self, id: i32) -> Result<(), CatalogError>;
    fn delete_tag(&self, tag: &NormalizedTag) -> Result<(), CatalogError>;
    fn rename_tag(
        &self,
        old_name: &NormalizedTag,
        new_name: &NormalizedTag,
    ) -> Result<(), CatalogError>;
    fn merge_tag(&self, source: &NormalizedTag, target: &NormalizedTag)
    -> Result<(), CatalogError>;
    /// Insert a new dataset record using the uid and metadata from an imported
    /// archive.  Tags in `metadata.tags` are created if they don't exist.
    fn insert_imported_dataset_record(
        &self,
        metadata: &ExportedMetadata,
    ) -> Result<DatasetRecord, CatalogError>;
    /// Replace an existing dataset record in place using metadata from an
    /// imported archive. The existing record id is preserved and trashed/deleted
    /// state is cleared.
    fn replace_imported_dataset_record(
        &self,
        id: i32,
        metadata: &ExportedMetadata,
    ) -> Result<DatasetRecord, CatalogError>;
    /// Find a dataset record by its uuid without requiring the DB id.
    fn find_dataset_by_uid(
        &self,
        uid: uuid::Uuid,
    ) -> Result<Option<DatasetRecord>, CatalogError>;
}
