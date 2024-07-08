use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chrono::NaiveDate;
use uuid::Uuid;

use crate::dir::WorkDirectory;

pub struct DataFile(File);

fn get_data_set_path(root: &WorkDirectory, date: NaiveDate, uid: Uuid) -> PathBuf {
    root.data_dir().join(format!("{}/{}", date, uid))
}

fn create_data_set(path: &Path) {}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_get_data_set_path() {
        let root = WorkDirectory::new(PathBuf::from("/tmp"));
        let date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let uid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = get_data_set_path(&root, date, uid);
        assert_eq!(
            path,
            PathBuf::from("/tmp/data/2021-01-01/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0")
        );
    }
}
