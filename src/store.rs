use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::model::Id;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Persist(#[from] tempfile::PersistError),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct KVStore {
    directory: PathBuf,
}

impl KVStore {
    pub fn new(directory: &Path) -> Result<Self, StoreError> {
        fs::create_dir_all(directory)?;
        Ok(Self {
            directory: directory.to_path_buf(),
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn file_path(&self, key: Id) -> PathBuf {
        self.directory.join(format!("{}.evdata", key.digest))
    }

    pub fn set(&self, key: Id, value: &[u8]) -> Result<(), StoreError> {
        let temp_file = NamedTempFile::new()?;

        fs::write(&temp_file, value)?;

        let file_path = self.file_path(key);

        temp_file.persist(&file_path)?;
        Ok(())
    }

    pub fn get(&self, key: Id) -> Result<Vec<u8>, StoreError> {
        let file_path = self.file_path(key);
        let data = fs::read(&file_path)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new() {
        let dir = tempdir().unwrap();
        let store_dir = dir.path().join("store");
        let store = KVStore::new(&store_dir).unwrap();

        assert!(store_dir.exists());
        assert_eq!(store.directory(), store_dir);
    }

    #[test]
    fn test_directory() {
        let dir = tempdir().unwrap();
        let store = KVStore::new(dir.path()).unwrap();

        assert_eq!(store.directory(), dir.path());
    }

    #[test]
    fn test_file_path() {
        let dir = tempdir().unwrap();
        let store = KVStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };

        let expected = dir.path().join("12345.evdata");
        assert_eq!(store.file_path(id), expected);
    }

    #[test]
    fn test_set() {
        let dir = tempdir().unwrap();
        let store = KVStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };
        let data = b"test data";

        store.set(id, data).unwrap();

        let file_path = store.file_path(id);
        let read_data = fs::read(file_path).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_get() {
        let dir = tempdir().unwrap();
        let store = KVStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };
        let data = b"test data";

        fs::write(store.file_path(id), data).unwrap();

        let read_data = store.get(id).unwrap();
        assert_eq!(read_data, data);
    }
}
