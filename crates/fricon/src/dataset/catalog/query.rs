use diesel::{SqliteConnection, prelude::*};
use tracing::instrument;

use crate::{
    DEFAULT_DATASET_LIST_LIMIT,
    dataset::{
        catalog::CatalogError,
        model::{
            DatasetId, DatasetListQuery, DatasetRecord, DatasetSortBy, DatasetStatus, SortDirection,
        },
        sqlite::{self, schema},
    },
};

#[instrument(skip(conn, id), fields(dataset.id = ?id))]
pub(crate) fn do_get_dataset(
    conn: &mut SqliteConnection,
    id: DatasetId,
) -> Result<DatasetRecord, CatalogError> {
    let dataset = match id {
        DatasetId::Id(dataset_id) => sqlite::Dataset::find_by_id(conn, dataset_id)?,
        DatasetId::Uid(uid) => sqlite::Dataset::find_by_uid(conn, uid)?,
    };

    let Some(dataset) = dataset else {
        let id_str = match id {
            DatasetId::Id(i) => i.to_string(),
            DatasetId::Uid(u) => u.to_string(),
        };
        return Err(CatalogError::NotFound { id: id_str });
    };

    let tags = dataset.load_tags(conn)?;

    Ok(sqlite::dataset_record_from_models(dataset, tags))
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
) -> Result<Option<Vec<i32>>, CatalogError> {
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

fn normalize_statuses(statuses: Option<&[DatasetStatus]>) -> Option<Vec<DatasetStatus>> {
    statuses.and_then(|statuses| {
        let mut deduped = statuses.to_vec();
        deduped.sort_unstable_by_key(|status| *status as u8);
        deduped.dedup();
        if deduped.is_empty() {
            None
        } else {
            Some(deduped)
        }
    })
}

fn map_datasets_with_tags(
    conn: &mut SqliteConnection,
    all_datasets: Vec<sqlite::Dataset>,
) -> Result<Vec<DatasetRecord>, CatalogError> {
    let dataset_tags = sqlite::DatasetTag::belonging_to(&all_datasets)
        .inner_join(schema::tags::table)
        .select((sqlite::DatasetTag::as_select(), sqlite::Tag::as_select()))
        .load::<(sqlite::DatasetTag, sqlite::Tag)>(conn)?;

    let datasets_with_tags: Vec<(sqlite::Dataset, Vec<sqlite::Tag>)> = dataset_tags
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
        .map(|(dataset, tags)| sqlite::dataset_record_from_models(dataset, tags))
        .collect())
}

#[instrument(skip(conn, query_options))]
pub(crate) fn do_list_datasets(
    conn: &mut SqliteConnection,
    query_options: &DatasetListQuery,
) -> Result<Vec<DatasetRecord>, CatalogError> {
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
    let all_datasets: Vec<sqlite::Dataset> = query
        .limit(limit)
        .offset(offset)
        .select(sqlite::Dataset::as_select())
        .load(conn)?;
    map_datasets_with_tags(conn, all_datasets)
}

#[instrument(skip(conn))]
pub(crate) fn do_list_dataset_tags(
    conn: &mut SqliteConnection,
) -> Result<Vec<String>, CatalogError> {
    let tags = schema::tags::table
        .select(schema::tags::name)
        .order(schema::tags::name.asc())
        .load(conn)?;
    Ok(tags)
}
