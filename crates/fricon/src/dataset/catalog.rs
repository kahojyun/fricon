mod error;
mod service;

pub use self::error::CatalogError;
pub(crate) use self::service::DatasetCatalogService;
use crate::dataset::{
    NormalizedTag,
    model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait DatasetCatalogRepository: Send + Sync {
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError>;
    fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError>;
    fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), CatalogError>;
    fn add_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError>;
    fn remove_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError>;
    fn delete_dataset(&self, id: i32) -> Result<(), CatalogError>;
    fn trash_dataset(&self, id: i32) -> Result<(), CatalogError>;
    fn restore_dataset(&self, id: i32) -> Result<(), CatalogError>;
    fn purge_trashed_datasets(&self) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn delete_tag(&self, tag: &NormalizedTag) -> Result<(), CatalogError>;
    fn rename_tag(
        &self,
        old_name: &NormalizedTag,
        new_name: &NormalizedTag,
    ) -> Result<(), CatalogError>;
    fn merge_tag(&self, source: &NormalizedTag, target: &NormalizedTag)
    -> Result<(), CatalogError>;
}
