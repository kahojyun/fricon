mod error;
mod service;

pub use self::{error::CatalogError, service::DatasetCatalogService};
use crate::dataset::model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait DatasetCatalogRepository: Send + Sync {
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError>;
    fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError>;
    fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError>;
    fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), CatalogError>;
    fn add_tags(&self, id: i32, tags: &[String]) -> Result<(), CatalogError>;
    fn remove_tags(&self, id: i32, tags: &[String]) -> Result<(), CatalogError>;
    fn delete_dataset(&self, id: i32) -> Result<(), CatalogError>;
}
