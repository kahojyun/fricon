use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::NaiveDate;
use log::info;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions,
};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug)]
pub struct WorkDirectory(PathBuf);

impl WorkDirectory {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn data_dir(&self) -> PathBuf {
        self.0.join("data")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.0.join("log")
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.0.join("backup")
    }

    pub fn config_path(&self) -> PathBuf {
        self.0.join("config.toml")
    }

    pub fn database_path(&self) -> PathBuf {
        self.0.join("fricon.sqlite3")
    }

    pub async fn init(&self) {
        info!("Initalize work directory: {:?}", self.0);
        self.ensure_empty_dir();
        self.init_config();
        self.init_database().await;
        self.init_dir();
    }

    pub fn check(&self) {
        if !self.0.is_dir() {
            panic!("Not a directory: {:?}", self.0);
        }
        if !self.config_path().is_file() {
            panic!("Missing configuration: {:?}", self.config_path());
        }
        if !self.database_path().is_file() {
            panic!("Missing database: {:?}", self.database_path());
        }
        if !self.data_dir().is_dir() {
            panic!("Missing data directory: {:?}", self.data_dir());
        }
        if !self.log_dir().is_dir() {
            panic!("Missing log directory: {:?}", self.log_dir());
        }
        if !self.backup_dir().is_dir() {
            panic!("Missing backup directory: {:?}", self.backup_dir());
        }
    }

    fn ensure_empty_dir(&self) {
        let path = &self.0;
        if path.is_dir() {
            if path
                .read_dir()
                .expect("Cannot open directory")
                .next()
                .is_some()
            {
                panic!("Directory is not empty: {:?}", path);
            }
            return;
        }
        info!("Create directory: {:?}", path);
        fs::create_dir_all(path).expect("Cannot create directory");
    }

    fn init_config(&self) {
        let config_path = self.config_path();
        info!("Initialize configuration: {:?}", config_path);
        let default_config = Config::default();
        let config_str = default_config.to_toml();
        fs::write(&config_path, config_str).expect("Cannot write configuration");
    }

    async fn init_database(&self) {
        let db_path = self.database_path();
        let db_url = format!("sqlite://{}", db_path.display());
        info!("Initialize database: {}", db_url);
        let mut conn = SqliteConnectOptions::from_str(&db_url)
            .expect("Cannot parse database URL")
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true)
            .connect()
            .await
            .expect("Cannot open database");
        MIGRATOR
            .run(&mut conn)
            .await
            .expect("Cannot run database migrations");
    }

    fn init_dir(&self) {
        fs::create_dir(self.data_dir()).expect("Cannot create data directory");
        fs::create_dir(self.log_dir()).expect("Cannot create log directory");
        fs::create_dir(self.backup_dir()).expect("Cannot create backup directory");
    }
}

#[derive(Debug)]
pub struct Workspace {
    root: WorkDirectory,
    config: Config,
}

pub static MIGRATOR: Migrator = sqlx::migrate!();

impl Workspace {
    pub fn open(path: PathBuf) -> Self {
        let root = WorkDirectory(path);
        root.check();
        let config_path = root.config_path();
        let config_str = fs::read_to_string(config_path).expect("Cannot read configuration");
        let config = Config::from_toml(&config_str).expect("Cannot parse configuration");
        Self { root, config }
    }

    pub fn root(&self) -> &WorkDirectory {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn get_dataset_path(&self, date: NaiveDate, uid: Uuid) -> PathBuf {
        get_dataset_path(&self.root.data_dir(), date, uid)
    }
}

fn get_dataset_path(data_dir: &Path, date: NaiveDate, uid: Uuid) -> PathBuf {
    data_dir.join(format!("{}/{}", date, uid))
}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_get_data_set_path() {
        let data_dir = PathBuf::from("/tmp/data");
        let date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let uid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = get_dataset_path(&data_dir, date, uid);
        assert_eq!(
            path,
            PathBuf::from("/tmp/data/2021-01-01/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0")
        );
    }
}
