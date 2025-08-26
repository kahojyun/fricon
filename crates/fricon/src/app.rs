use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use chrono::Local;
use deadpool_diesel::sqlite::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::info;
use uuid::Uuid;

use crate::{
    database::{
        self, DatasetStatus, DatasetTag, JsonValue, NewDataset, NewTag, PoolExt as _, SimpleUuid,
        Tag, schema,
    },
    dataset::{self, Dataset},
    dataset_manager::DatasetManager,
    server,
    workspace::WorkspaceRoot,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppEvent {
    DatasetCreated {
        id: i32,
        uuid: String,
        name: String,
        description: String,
        tags: Vec<String>,
    },
}

/// `AppState` contains only data - no business logic
/// This struct is cheaply cloneable and holds all the shared state
/// Internal-only, not exposed in public API
#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    root: WorkspaceRoot,
    database: Pool,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
    event_sender: broadcast::Sender<AppEvent>,
}

impl AppState {
    async fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let root = WorkspaceRoot::open(path)?;
        let db_path = root.paths().database_file();
        let backup_path = root
            .paths()
            .database_backup_file(Local::now().naive_local());
        let database = database::connect(db_path, backup_path).await?;
        let shutdown_token = CancellationToken::new();
        let tracker = TaskTracker::new();
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            inner: Arc::new(AppStateInner {
                root,
                database,
                shutdown_token,
                tracker,
                event_sender,
            }),
        })
    }

    #[must_use]
    fn root(&self) -> &WorkspaceRoot {
        &self.inner.root
    }

    #[must_use]
    fn database(&self) -> &Pool {
        &self.inner.database
    }

    #[must_use]
    fn tracker(&self) -> &TaskTracker {
        &self.inner.tracker
    }

    #[must_use]
    fn shutdown_token(&self) -> &CancellationToken {
        &self.inner.shutdown_token
    }

    #[must_use]
    fn event_sender(&self) -> &broadcast::Sender<AppEvent> {
        &self.inner.event_sender
    }

    fn subscribe_to_events(&self) -> broadcast::Receiver<AppEvent> {
        self.inner.event_sender.subscribe()
    }
}

/// `AppHandle` provides business logic methods
/// All dataset operations are implemented here
#[derive(Clone)]
pub struct AppHandle {
    state: AppState,
}

impl AppHandle {
    fn new(state: AppState) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn root(&self) -> &WorkspaceRoot {
        self.state.root()
    }

    #[must_use]
    pub fn database(&self) -> &Pool {
        self.state.database()
    }

    #[must_use]
    pub fn tracker(&self) -> &TaskTracker {
        self.state.tracker()
    }

    #[must_use]
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<AppEvent> {
        self.state.subscribe_to_events()
    }

    /// Create a `DatasetManager` for this app instance
    #[must_use]
    pub fn dataset_manager(&self) -> DatasetManager {
        DatasetManager::new(self.clone())
    }

    #[deprecated(note = "Use DatasetManager::create_dataset instead")]
    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        index_columns: Vec<String>,
    ) -> Result<dataset::Writer> {
        let uuid = Uuid::new_v4();
        let state = self.state.clone();
        let (dataset, tags) = state
            .database()
            .interact(move |conn| {
                conn.immediate_transaction(|conn| {
                    let new_dataset = NewDataset {
                        uuid: SimpleUuid(uuid),
                        name: &name,
                        description: &description,
                        status: DatasetStatus::Pending,
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
        let writer = Dataset::create(self.clone(), dataset.clone(), tags.clone())
            .context("Failed to create dataset.")?;

        // Send event notification
        let event = AppEvent::DatasetCreated {
            id: dataset.id,
            uuid: dataset.uuid.0.simple().to_string(),
            name: dataset.name.clone(),
            description: dataset.description.clone(),
            tags: tags.into_iter().map(|t| t.name).collect(),
        };
        let _ = self.state.event_sender().send(event);

        Ok(writer)
    }

    #[deprecated(note = "Use DatasetManager::get_dataset instead")]
    pub async fn get_dataset(&self, id: i32) -> Result<Dataset> {
        let (dataset, tags) = self
            .state
            .database()
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
        Ok(Dataset::new(self.clone(), dataset, tags))
    }

    #[deprecated(note = "Use DatasetManager::get_dataset with DatasetId::Uuid instead")]
    pub async fn get_dataset_by_uuid(&self, uuid: Uuid) -> Result<Dataset> {
        let (dataset, tags) = self
            .state
            .database()
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
        Ok(Dataset::new(self.clone(), dataset, tags))
    }

    #[deprecated(note = "Use DatasetManager::list_datasets instead")]
    pub async fn list_datasets(&self) -> Result<Vec<(database::Dataset, Vec<database::Tag>)>> {
        self.state
            .database()
            .interact(|conn| {
                let all_datasets = schema::datasets::table
                    .order(schema::datasets::id.desc())
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
                    .map(|(dataset_tags, dataset)| {
                        (
                            dataset,
                            dataset_tags.into_iter().map(|(_, tag)| tag).collect(),
                        )
                    })
                    .collect();

                Ok(datasets_with_tags)
            })
            .await
    }
}

/// `AppManager` manages the application lifecycle
/// Responsible for initialization, server management, and shutdown
pub struct AppManager {
    state: AppState,
    handle: AppHandle,
}

impl AppManager {
    pub async fn serve(path: impl Into<PathBuf>) -> Result<Self> {
        let state = AppState::new(path).await?;
        let handle = AppHandle::new(state.clone());

        // Start the server
        state
            .tracker()
            .spawn(server::run(handle.clone(), state.shutdown_token().clone()));

        Ok(Self { state, handle })
    }

    pub async fn shutdown(&self) {
        self.state.shutdown_token().cancel();
        self.state.tracker().close();
        self.state.tracker().wait().await;
    }

    #[must_use]
    pub fn handle(&self) -> &AppHandle {
        &self.handle
    }
}
