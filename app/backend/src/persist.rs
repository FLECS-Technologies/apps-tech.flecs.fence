pub mod group_db;
pub mod user_db;

use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use group_db::GroupDB;
use user_db::UserDB;

pub struct Db {
    pub groups: GroupDB,
    pub users: UserDB,
}

impl Default for Db {
    fn default() -> Self {
        Self {
            groups: GroupDB::new("/var/local/lib/fence/groups.json".into()).unwrap(),
            users: UserDB::new("/var/local/lib/fence/users.json".into()).unwrap(),
        }
    }
}

pub fn load_from_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    let try_load_from_file = |path: &Path| -> Result<T> {
        let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)
            .with_context(|| format!("deserialize data from {}", path.display()))?;
        Ok(data)
    };

    match try_load_from_file(path) {
        Ok(data) => Ok(data),
        Err(e1) => {
            let backup_path = path.with_extension("bak");
            match try_load_from_file(&backup_path) {
                Ok(data) => Ok(data),
                Err(e2) => {
                    let is_io_error = |e: &anyhow::Error| {
                        e.downcast_ref::<std::io::Error>()
                            .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                    };
                    let no_files_found = is_io_error(&e1) && is_io_error(&e2);
                    if no_files_found {
                        Ok(T::default())
                    } else {
                        Err(e1)
                    }
                }
            }
        }
    }
}

pub fn save_to_file<T>(path: &Path, data: &T) -> Result<()>
where
    T: Serialize,
{
    /* Attempt to create directory */
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory '{}'", parent.display()))?
    }

    /* Backup existing file, if possible */
    if path.exists() {
        let backup = path.with_extension("bak");
        if let Err(e) = fs::copy(path, &backup) {
            eprintln!(
                "backup '{}' to '{}': {}",
                path.display(),
                backup.display(),
                e
            );
        }
    }

    /* Write data to temporary file */
    let tmp = path.with_extension("tmp");
    let mut tmp_file = fs::File::create(&tmp)
        .with_context(|| format!("create temporary file '{}'", tmp.display()))?;
    serde_json::to_writer_pretty(&mut tmp_file, data)
        .with_context(|| format!("serialize data to '{}'", tmp.display()))?;
    tmp_file
        .flush()
        .with_context(|| format!("flush contents of {}", tmp.display()))?;

    /* Rename temporary file */
    fs::rename(&tmp, path)
        .with_context(|| format!("rename {} to {}", tmp.display(), path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};
    use std::sync::LazyLock;
    use tempfile::tempdir;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        data: (u32, String),
    }
    /* OnceLock */
    static TEST_DATA_1: LazyLock<Vec<TestData>> = LazyLock::new(|| {
        vec![
            TestData {
                id: 0,
                name: "test1".to_owned(),
                data: (0, "test1".to_owned()),
            },
            TestData {
                id: 1,
                name: "test2".to_owned(),
                data: (1, "test2".to_owned()),
            },
        ]
    });

    static TEST_DATA_2: LazyLock<Vec<TestData>> = LazyLock::new(|| {
        vec![
            TestData {
                id: 2,
                name: "test2".to_owned(),
                data: (2, "test2".to_owned()),
            },
            TestData {
                id: 3,
                name: "test3".to_owned(),
                data: (3, "test3".to_owned()),
            },
        ]
    });

    fn build_test_paths() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempdir().unwrap();
        let json_path = tmp.path().join("testdata.json");
        let backup_path = json_path.with_extension("bak");
        (tmp, json_path, backup_path)
    }

    #[test]
    fn save_and_load_ok() {
        let (_, path, backup_path) = build_test_paths();

        let data: Vec<TestData> =
            super::load_from_file(&path).expect("Load should succeed if file does not exist yet");
        assert_eq!(data, vec![]);

        super::save_to_file(&path, &*TEST_DATA_1).expect("Save to file w/o backup should succeed");
        assert!(path.exists() && path.is_file());
        assert!(!backup_path.exists());
        let data: Vec<TestData> =
            super::load_from_file(&path).expect("Load from file w/o backup should succeed");
        assert_eq!(data, *TEST_DATA_1);

        super::save_to_file(&path, &*TEST_DATA_2).expect("Save to file w/backup should succeed");
        assert!(path.exists() && path.is_file());
        assert!(backup_path.exists() && backup_path.is_file());
        let data: Vec<TestData> =
            super::load_from_file(&path).expect("Load from file w/backup should succeed");
        assert_eq!(data, *TEST_DATA_2);
    }

    #[test]
    fn save_to_file_not_ok() {
        use std::fs::{self, Permissions};
        use std::os::unix::fs::PermissionsExt;
        let (tmp, path, _) = build_test_paths();
        fs::set_permissions(tmp.path(), Permissions::from_mode(0o400)).unwrap();

        super::save_to_file(&path, &*TEST_DATA_1).expect_err("Save to file w/o backup should fail");
    }

    #[test]
    fn load_from_file_ok() {
        let (_, path, backup_path) = build_test_paths();

        super::save_to_file(&path, &*TEST_DATA_1).unwrap();
        super::save_to_file(&path, &*TEST_DATA_2).unwrap();
        assert!(path.exists() && path.is_file());
        assert!(backup_path.exists() && backup_path.is_file());

        let data: Vec<TestData> =
            super::load_from_file(&path).expect("Load from file should succeed");
        assert_eq!(data, *TEST_DATA_2);
    }

    #[test]
    fn load_from_backup_file_ok() {
        use std::fs;

        let (_, path, backup_path) = build_test_paths();

        super::save_to_file(&path, &*TEST_DATA_1).unwrap();
        super::save_to_file(&path, &*TEST_DATA_1).unwrap();
        fs::remove_file(&path).unwrap();
        assert!(!path.exists());
        assert!(backup_path.exists() && backup_path.is_file());

        let data: Vec<TestData> =
            super::load_from_file(&path).expect("Load from backup file should succeed");
        assert_eq!(data, *TEST_DATA_1);
    }
}
