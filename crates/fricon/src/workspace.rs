use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, ensure};
use deadpool_diesel::sqlite::Pool;
use diesel::prelude::*;
use semver::{Version, VersionReq};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    VERSION,
    database::{
        self, DatasetTag, JsonValue, NewDataset, NewTag, PoolExt as _, SimpleUuid, Tag, schema,
    },
    dataset::{self, Dataset},
    paths::WorkspacePath,
};

pub async fn init(path: &Path) -> Result<()> {
    info!("Initialize workspace: {:?}", path);
    create_empty_dir(path)?;
    let root = WorkspacePath::new(path)?;
    database::connect(root.database_file()).await?;
    init_dir(&root)?;
    write_version_file(&root.version_file())?;
    Ok(())
}

#[derive(Clone)]
pub struct Workspace(Arc<Shared>);

impl Workspace {
    pub async fn open(path: &Path) -> Result<Self> {
        let shared = Shared::open(path).await?;
        Ok(Self(Arc::new(shared)))
    }

    #[must_use]
    pub fn root(&self) -> &WorkspacePath {
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
    root: WorkspacePath,
    database: Pool,
    _lock: FileLock,
}

impl Shared {
    pub async fn open(path: &Path) -> Result<Self> {
        let root = WorkspacePath::new(path)?;
        let lock = FileLock::new(root.lock_file())?;
        check_version_file(&root.version_file())?;
        let database = database::connect(root.database_file()).await?;
        Ok(Self {
            root,
            database,
            _lock: lock,
        })
    }

    #[must_use]
    pub const fn root(&self) -> &WorkspacePath {
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

fn create_empty_dir(path: &Path) -> Result<()> {
    // Success if path already exists.
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    let mut dir_contents = path
        .read_dir()
        .with_context(|| format!("Failed to read directory contents: {}", path.display()))?;
    ensure!(
        dir_contents.next().is_none(),
        "Directory is not empty: {:?}",
        path
    );
    Ok(())
}

fn init_dir(root: &WorkspacePath) -> Result<()> {
    fs::create_dir(root.data_dir()).context("Failed to create data directory.")?;
    fs::create_dir(root.log_dir()).context("Failed to create log directory.")?;
    fs::create_dir(root.backup_dir()).context("Failed to create backup directory.")?;
    Ok(())
}

fn write_version_file(path: &Path) -> Result<()> {
    fs::write(path, format!("{VERSION}\n")).context("Failed to write version file.")?;
    Ok(())
}

fn check_version_file(path: &Path) -> Result<()> {
    let version_str = fs::read_to_string(path).context("Failed to read workspace version file.")?;
    let workspace_version =
        Version::parse(version_str.trim()).context("Failed to parse version.")?;
    let req =
        VersionReq::parse(&format!("={VERSION}")).expect("Failed to parse version requirement.");
    ensure!(
        req.matches(&workspace_version),
        "Version mismatch: {} != {}",
        VERSION,
        workspace_version
    );
    Ok(())
}

#[derive(Debug)]
struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .context("Failed to open file for locking.")?;
        file.try_lock().context("Failed to acquire file lock.")?;
        Ok(Self { file, path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            warn!("Failed to release file lock: {e}");
        }
        if let Err(e) = fs::remove_file(&self.path) {
            warn!("Failed to remove locked file: {e}");
        }
    }
}
