use crate::dataset_catalog::tasks;

pub(super) fn get_dataset(
    conn: &mut diesel::SqliteConnection,
    id: crate::dataset_catalog::DatasetId,
) -> Result<crate::dataset_catalog::DatasetRecord, crate::dataset_catalog::DatasetCatalogError> {
    tasks::do_get_dataset(conn, id)
}

pub(super) fn list_datasets(
    conn: &mut diesel::SqliteConnection,
    query: &crate::dataset_catalog::DatasetListQuery,
) -> Result<Vec<crate::dataset_catalog::DatasetRecord>, crate::dataset_catalog::DatasetCatalogError>
{
    tasks::do_list_datasets(conn, query)
}

pub(super) fn list_dataset_tags(
    conn: &mut diesel::SqliteConnection,
) -> Result<Vec<String>, crate::dataset_catalog::DatasetCatalogError> {
    tasks::do_list_dataset_tags(conn)
}
