use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use chrono::Local;
use deadpool_diesel::sqlite::Pool;
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::{
    database::{
        self, DatasetTag, JsonValue, NewDataset, NewTag, PoolExt as _, SimpleUuid, Tag, schema,
    },
    dataset::{self, Dataset},
    paths::WorkspaceRoot,
};

pub async fn init(path: impl Into<PathBuf>) -> Result<()> {
    let path = path.into();
    info!("Initialize workspace: {}", path.display());
    let root = WorkspaceRoot::init(path)?;
    let db_path = root.paths().database_file();
    let backup_path = root
        .paths()
        .database_backup_file(Local::now().naive_local());
    database::connect(db_path, backup_path).await?;
    Ok(())
}

#[derive(Clone)]
pub struct Workspace(Arc<Shared>);

impl Workspace {
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let shared = Shared::open(path).await?;
        Ok(Self(Arc::new(shared)))
    }

    #[must_use]
    pub fn root(&self) -> &WorkspaceRoot {
        self.0.root()
    }

    #[must_use]
    pub fn database(&self) -> &Pool {
        self.0.database()
    }

    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        index_columns: Vec<String>,
    ) -> Result<dataset::Writer> {
        self.0
            .clone()
            .create_dataset(name, description, tags, index_columns)
            .await
    }

    pub async fn get_dataset(&self, id: i32) -> Result<Dataset> {
        self.0.clone().get_dataset(id).await
    }

    pub async fn get_dataset_by_uuid(&self, uuid: Uuid) -> Result<Dataset> {
        self.0.clone().get_dataset_by_uuid(uuid).await
    }

    pub async fn list_datasets(&self) -> Result<Vec<(database::Dataset, Vec<database::Tag>)>> {
        self.0.list_datasets().await
    }
}

struct Shared {
    root: WorkspaceRoot,
    database: Pool,
}

impl Shared {
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let root = WorkspaceRoot::open(path)?;
        let db_path = root.paths().database_file();
        let backup_path = root
            .paths()
            .database_backup_file(Local::now().naive_local());
        let database = database::connect(db_path, backup_path).await?;
        Ok(Self { root, database })
    }

    #[must_use]
    pub fn root(&self) -> &WorkspaceRoot {
        &self.root
    }

    #[must_use]
    pub fn database(&self) -> &Pool {
        &self.database
    }

    pub async fn create_dataset(
        self: Arc<Self>,
        name: String,
        description: String,
        tags: Vec<String>,
        index_columns: Vec<String>,
    ) -> Result<dataset::Writer> {
        let uuid = Uuid::new_v4();
        let (dataset, tags) = self
            .database
            .interact(move |conn| {
                conn.immediate_transaction(|conn| {
                    let new_dataset = NewDataset {
                        uuid: SimpleUuid(uuid),
                        name: &name,
                        description: &description,
                        index_columns: JsonValue(&index_columns),
                    };
                    let dataset = diesel::insert_into(schema::datasets::table)
                        .values(new_dataset)
                        .returning(database::Dataset::as_returning())
                        .get_result(conn)?;
                    let new_tags = tags
                        .iter()
                        .map(|tag| NewTag { name: tag })
                        .collect::<Vec<_>>();
                    diesel::insert_or_ignore_into(schema::tags::table)
                        .values(new_tags)
                        .execute(conn)?;
                    let tags = schema::tags::table
                        .filter(schema::tags::name.eq_any(&tags))
                        .load::<Tag>(conn)?;
                    let dataset_tags: Vec<_> = tags
                        .iter()
                        .map(|tag| DatasetTag {
                            dataset_id: dataset.id,
                            tag_id: tag.id,
                        })
                        .collect();
                    diesel::insert_into(schema::datasets_tags::table)
                        .values(dataset_tags)
                        .execute(conn)?;
                    Ok((dataset, tags))
                })
            })
            .await?;
        let writer =
            Dataset::create(Workspace(self), dataset, tags).context("Failed to create dataset.")?;
        Ok(writer)
    }

    pub async fn list_datasets(&self) -> Result<Vec<(database::Dataset, Vec<database::Tag>)>> {
        self.database
            .interact(|conn| {
                let all_datasets = schema::datasets::table
                    .select(database::Dataset::as_select())
                    .load(conn)?;

                let dataset_tags = database::DatasetTag::belonging_to(&all_datasets)
                    .inner_join(schema::tags::table)
                    .select((
                        database::DatasetTag::as_select(),
                        database::Tag::as_select(),
                    ))
                    .load::<(database::DatasetTag, database::Tag)>(conn)?;

                let datasets_with_tags: Vec<(database::Dataset, Vec<database::Tag>)> = dataset_tags
                    .grouped_by(&all_datasets)
                    .into_iter()
                    .zip(all_datasets)
                    .map(|(dt, dataset)| (dataset, dt.into_iter().map(|(_, tag)| tag).collect()))
                    .collect();

                Ok(datasets_with_tags)
            })
            .await
    }

    pub async fn get_dataset(self: Arc<Self>, id: i32) -> Result<Dataset> {
        let (dataset, tags) = self
            .database
            .interact(move |conn| {
                let dataset = schema::datasets::table
                    .find(id)
                    .select(database::Dataset::as_select())
                    .first(conn)?;
                let tags = database::DatasetTag::belonging_to(&dataset)
                    .inner_join(schema::tags::table)
                    .select(database::Tag::as_select())
                    .load(conn)?;
                Ok((dataset, tags))
            })
            .await?;
        Ok(Dataset::new(Workspace(self), dataset, tags))
    }

    pub async fn get_dataset_by_uuid(self: Arc<Self>, uuid: Uuid) -> Result<Dataset> {
        let (dataset, tags) = self
            .database
            .interact(move |conn| {
                let dataset = schema::datasets::table
                    .filter(schema::datasets::uuid.eq(uuid.as_simple().to_string()))
                    .select(database::Dataset::as_select())
                    .first(conn)?;
                let tags = database::DatasetTag::belonging_to(&dataset)
                    .inner_join(schema::tags::table)
                    .select(database::Tag::as_select())
                    .load(conn)?;
                Ok((dataset, tags))
            })
            .await?;
        Ok(Dataset::new(Workspace(self), dataset, tags))
    }
}
