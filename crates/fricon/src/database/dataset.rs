mod models;
mod types;

use diesel::{SqliteConnection, prelude::*};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use self::{
    models::{Dataset, DatasetTag, DatasetUpdate as DbDatasetUpdate, NewDataset, Tag},
    types::{DbDatasetStatus, SimpleUuid},
};
use crate::{
    DEFAULT_DATASET_LIST_LIMIT,
    database::{core::Pool, schema},
    dataset::{
        NormalizedTag,
        catalog::{CatalogError, DatasetCatalogRepository},
        ingest::{CreateDatasetRequest, DatasetIngestRepository, IngestError},
        model::{
            DatasetId, DatasetListQuery, DatasetMetadata, DatasetRecord, DatasetSortBy,
            DatasetStatus, DatasetUpdate, SortDirection,
        },
        read::{DatasetLocation, DatasetReadRepository, ReadError},
    },
};

#[derive(Clone)]
pub(crate) struct DatasetRepository {
    pool: Pool,
}

impl DatasetRepository {
    #[must_use]
    pub(crate) fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

fn dataset_record_from_models(dataset: Dataset, tags: Vec<Tag>) -> DatasetRecord {
    let metadata = DatasetMetadata {
        uid: dataset.uid.0,
        name: dataset.name,
        description: dataset.description,
        favorite: dataset.favorite,
        status: dataset.status.0,
        created_at: dataset.created_at.and_utc(),
        trashed_at: dataset.trashed_at.map(|value| value.and_utc()),
        tags: tags.into_iter().map(|tag| tag.name).collect(),
    };

    DatasetRecord {
        id: dataset.id,
        metadata,
    }
}

fn dataset_location_from_model(dataset: &Dataset) -> DatasetLocation {
    DatasetLocation {
        id: dataset.id,
        uid: dataset.uid.0,
    }
}

fn get_dataset_model(
    conn: &mut SqliteConnection,
    id: DatasetId,
) -> diesel::QueryResult<Option<Dataset>> {
    match id {
        DatasetId::Id(dataset_id) => Dataset::find_by_id(conn, dataset_id),
        DatasetId::Uid(uid) => Dataset::find_by_uid(conn, uid),
    }
}

fn dataset_not_found(id: DatasetId) -> String {
    match id {
        DatasetId::Id(value) => value.to_string(),
        DatasetId::Uid(value) => value.to_string(),
    }
}

fn normalize_search(search: Option<&str>) -> Option<&str> {
    search.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_tag_filters(tags: Option<&[String]>) -> Option<Vec<String>> {
    tags.and_then(|tags| {
        let cleaned: Vec<String> = tags
            .iter()
            .map(|tag| tag.trim())
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    })
}

fn resolve_tagged_dataset_ids(
    conn: &mut SqliteConnection,
    tag_filters: Option<&[String]>,
) -> Result<Option<Vec<i32>>, anyhow::Error> {
    let Some(tag_filters) = tag_filters else {
        return Ok(None);
    };

    let ids = schema::datasets_tags::table
        .inner_join(schema::tags::table)
        .filter(schema::tags::name.eq_any(tag_filters))
        .select(schema::datasets_tags::dataset_id)
        .distinct()
        .load::<i32>(conn)?;

    if ids.is_empty() {
        Ok(Some(Vec::new()))
    } else {
        Ok(Some(ids))
    }
}

fn normalize_statuses(statuses: Option<&[DatasetStatus]>) -> Option<Vec<DbDatasetStatus>> {
    statuses.and_then(|statuses| {
        let mut deduped = statuses.to_vec();
        deduped.sort_unstable_by_key(|status| *status as u8);
        deduped.dedup();
        if deduped.is_empty() {
            None
        } else {
            Some(deduped.into_iter().map(DbDatasetStatus::from).collect())
        }
    })
}

fn apply_trashed_filter<'a>(
    query: schema::datasets::BoxedQuery<'a, diesel::sqlite::Sqlite>,
    trashed: Option<bool>,
) -> schema::datasets::BoxedQuery<'a, diesel::sqlite::Sqlite> {
    match trashed {
        Some(true) => query.filter(schema::datasets::trashed_at.is_not_null()),
        Some(false) => query.filter(schema::datasets::trashed_at.is_null()),
        None => query,
    }
}

fn map_datasets_with_tags(
    conn: &mut SqliteConnection,
    all_datasets: Vec<Dataset>,
) -> Result<Vec<DatasetRecord>, anyhow::Error> {
    let dataset_tags = DatasetTag::belonging_to(&all_datasets)
        .inner_join(schema::tags::table)
        .select((DatasetTag::as_select(), Tag::as_select()))
        .load::<(DatasetTag, Tag)>(conn)?;

    let datasets_with_tags: Vec<(Dataset, Vec<Tag>)> = dataset_tags
        .grouped_by(&all_datasets)
        .into_iter()
        .zip(all_datasets)
        .map(|(dataset_tags, dataset)| {
            (
                dataset,
                dataset_tags.into_iter().map(|(_, tag)| tag).collect(),
            )
        })
        .collect();

    Ok(datasets_with_tags
        .into_iter()
        .map(|(dataset, tags)| dataset_record_from_models(dataset, tags))
        .collect())
}

#[instrument(skip(conn, query_options))]
fn list_dataset_records(
    conn: &mut SqliteConnection,
    query_options: &DatasetListQuery,
) -> Result<Vec<DatasetRecord>, anyhow::Error> {
    let search = normalize_search(query_options.search.as_deref());
    let tag_filters = normalize_tag_filters(query_options.tags.as_deref());
    let tagged_dataset_ids = resolve_tagged_dataset_ids(conn, tag_filters.as_deref())?;
    if tagged_dataset_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }
    let statuses = normalize_statuses(query_options.statuses.as_deref());

    let mut query = schema::datasets::table.into_boxed();
    if let Some(search) = search {
        let pattern = format!("%{search}%");
        query = query.filter(schema::datasets::name.like(pattern));
    }
    if let Some(ids) = tagged_dataset_ids {
        query = query.filter(schema::datasets::id.eq_any(ids));
    }
    if query_options.favorite_only {
        query = query.filter(schema::datasets::favorite.eq(true));
    }
    if let Some(statuses) = statuses {
        query = query.filter(schema::datasets::status.eq_any(statuses));
    }
    query = apply_trashed_filter(query, query_options.trashed);

    query = match (query_options.sort_by, query_options.sort_direction) {
        (DatasetSortBy::Id, SortDirection::Asc) => query.order(schema::datasets::id.asc()),
        (DatasetSortBy::Id, SortDirection::Desc) => query.order(schema::datasets::id.desc()),
        (DatasetSortBy::Name, SortDirection::Asc) => {
            query.order((schema::datasets::name.asc(), schema::datasets::id.desc()))
        }
        (DatasetSortBy::Name, SortDirection::Desc) => {
            query.order((schema::datasets::name.desc(), schema::datasets::id.desc()))
        }
        (DatasetSortBy::CreatedAt, SortDirection::Asc) => query.order((
            schema::datasets::created_at.asc(),
            schema::datasets::id.desc(),
        )),
        (DatasetSortBy::CreatedAt, SortDirection::Desc) => query.order((
            schema::datasets::created_at.desc(),
            schema::datasets::id.desc(),
        )),
    };

    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_DATASET_LIST_LIMIT)
        .max(0);
    let offset = query_options.offset.unwrap_or(0).max(0);
    let all_datasets: Vec<Dataset> = query
        .limit(limit)
        .offset(offset)
        .select(Dataset::as_select())
        .load(conn)?;

    map_datasets_with_tags(conn, all_datasets)
}

#[instrument(skip(conn, id), fields(dataset.id = ?id))]
fn get_dataset_record(
    conn: &mut SqliteConnection,
    id: DatasetId,
) -> Result<Option<DatasetRecord>, anyhow::Error> {
    let Some(dataset) = get_dataset_model(conn, id)? else {
        return Ok(None);
    };
    let tags = dataset.load_tags(conn)?;
    Ok(Some(dataset_record_from_models(dataset, tags)))
}

#[instrument(skip(conn, id), fields(dataset.id = ?id))]
fn get_dataset_location(
    conn: &mut SqliteConnection,
    id: DatasetId,
) -> Result<Option<DatasetLocation>, anyhow::Error> {
    let Some(dataset) = get_dataset_model(conn, id)? else {
        return Ok(None);
    };
    Ok(Some(dataset_location_from_model(&dataset)))
}

pub(crate) fn cleanup_writing_datasets(pool: &Pool) -> Result<usize, anyhow::Error> {
    use schema::datasets::dsl::{datasets, status};

    let mut conn = pool.get()?;
    let updated_count =
        diesel::update(datasets.filter(status.eq(DbDatasetStatus::from(DatasetStatus::Writing))))
            .set(status.eq(DbDatasetStatus::from(DatasetStatus::Aborted)))
            .execute(&mut conn)?;

    if updated_count > 0 {
        info!(
            "Updated {} datasets from 'writing' to 'aborted' status",
            updated_count
        );
    }

    Ok(updated_count)
}

impl DatasetCatalogRepository for DatasetRepository {
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let record = get_dataset_record(&mut conn, id).map_err(CatalogError::from)?;
        record.ok_or_else(|| CatalogError::NotFound {
            id: dataset_not_found(id),
        })
    }

    fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        list_dataset_records(&mut conn, &query_options).map_err(CatalogError::from)
    }

    fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let tags = schema::tags::table
            .select(schema::tags::name)
            .order(schema::tags::name.asc())
            .load(&mut conn)
            .map_err(anyhow::Error::from)?;
        Ok(tags)
    }

    fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let db_update = DbDatasetUpdate {
            name: update.name,
            description: update.description,
            favorite: update.favorite,
            status: None,
            trashed_at: None,
        };
        Dataset::update_metadata(&mut conn, id, &db_update).map_err(anyhow::Error::from)?;
        debug!(dataset.id = id, "Dataset metadata updated");
        Ok(())
    }

    fn add_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let tag_names: Vec<String> = tags.iter().map(|tag| tag.as_str().to_string()).collect();
        conn.immediate_transaction(|conn| {
            let created_tags = Tag::find_or_create_batch(conn, &tag_names)?;
            let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();
            DatasetTag::create_associations(conn, id, &tag_ids)?;
            Ok::<(), diesel::result::Error>(())
        })
        .map_err(anyhow::Error::from)?;
        debug!(dataset.id = id, ?tags, "Tags added to dataset");
        Ok(())
    }

    fn remove_tags(&self, id: i32, tags: &[NormalizedTag]) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let tag_names: Vec<String> = tags.iter().map(|tag| tag.as_str().to_string()).collect();
        conn.immediate_transaction(|conn| {
            let tag_ids_to_delete = schema::tags::table
                .filter(schema::tags::name.eq_any(tag_names))
                .select(schema::tags::id)
                .load::<i32>(conn)?;

            DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
            Ok::<(), diesel::result::Error>(())
        })
        .map_err(anyhow::Error::from)?;
        debug!(dataset.id = id, ?tags, "Tags removed from dataset");
        Ok(())
    }

    fn delete_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        Dataset::delete_from_db(&mut conn, id).map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn trash_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        Dataset::trash(&mut conn, id).map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn restore_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        Dataset::restore(&mut conn, id).map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn purge_trashed_datasets(&self) -> Result<Vec<DatasetRecord>, CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let trashed_datasets = list_dataset_records(
            &mut conn,
            &DatasetListQuery {
                trashed: Some(true),
                ..DatasetListQuery::default()
            },
        )
        .map_err(CatalogError::from)?;
        Dataset::delete_trashed(&mut conn).map_err(anyhow::Error::from)?;
        Ok(trashed_datasets)
    }

    fn delete_tag(&self, tag: &NormalizedTag) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        conn.immediate_transaction(|conn| Tag::delete_by_name(conn, tag.as_str()))
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn rename_tag(
        &self,
        old_name: &NormalizedTag,
        new_name: &NormalizedTag,
    ) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        conn.immediate_transaction(|conn| Tag::rename(conn, old_name.as_str(), new_name.as_str()))
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn merge_tag(
        &self,
        source: &NormalizedTag,
        target: &NormalizedTag,
    ) -> Result<(), CatalogError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        conn.immediate_transaction(|conn| Tag::merge_into(conn, source.as_str(), target.as_str()))
            .map_err(anyhow::Error::from)?;
        Ok(())
    }
}

impl DatasetIngestRepository for DatasetRepository {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<DatasetRecord, IngestError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let (dataset, tags) = create_dataset_db_record(&mut conn, request, uid)?;
        Ok(dataset_record_from_models(dataset, tags))
    }

    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), IngestError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        Dataset::update_status(&mut conn, id, DbDatasetStatus::from(status))
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, IngestError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let record = get_dataset_record(&mut conn, id).map_err(IngestError::from)?;
        record.ok_or_else(|| IngestError::NotFound {
            id: dataset_not_found(id),
        })
    }
}

impl DatasetReadRepository for DatasetRepository {
    fn resolve_dataset(&self, id: DatasetId) -> Result<DatasetLocation, ReadError> {
        let mut conn = self.pool.get().map_err(anyhow::Error::from)?;
        let location = get_dataset_location(&mut conn, id).map_err(ReadError::from)?;
        location.ok_or_else(|| ReadError::NotFound {
            id: dataset_not_found(id),
        })
    }
}

fn create_dataset_db_record(
    conn: &mut SqliteConnection,
    request: &CreateDatasetRequest,
    uid: Uuid,
) -> Result<(Dataset, Vec<Tag>), anyhow::Error> {
    conn.immediate_transaction(|conn| {
        let new_dataset = NewDataset {
            uid: SimpleUuid(uid),
            name: &request.name,
            description: &request.description,
            status: DbDatasetStatus::from(DatasetStatus::Writing),
        };

        let dataset = diesel::insert_into(schema::datasets::table)
            .values(new_dataset)
            .returning(Dataset::as_returning())
            .get_result(conn)?;

        let tags = if request.tags.is_empty() {
            vec![]
        } else {
            let created_tags = Tag::find_or_create_batch(conn, &request.tags)?;
            let tag_ids: Vec<i32> = created_tags.iter().map(|tag| tag.id).collect();
            DatasetTag::create_associations(conn, dataset.id, &tag_ids)?;
            created_tags
        };

        Ok::<(Dataset, Vec<Tag>), diesel::result::Error>((dataset, tags))
    })
    .map_err(anyhow::Error::from)
}
