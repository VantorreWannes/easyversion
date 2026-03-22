use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;
use walkdir::WalkDir;

use crate::model::Id;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Persist(#[from] tempfile::PersistError),
}

/// An atomic, content-addressed file store.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FileStore {
    directory: PathBuf,
}

impl FileStore {
    /// Initializes a new `FileStore` at the specified path, creating the directory if it does not exist.
    pub fn new(directory: &Path) -> Result<Self, StoreError> {
        fs::create_dir_all(directory)?;
        Ok(Self {
            directory: directory.to_path_buf(),
        })
    }

    /// Returns the absolute path to the directory underlying this store.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn file_path(&self, key: Id) -> PathBuf {
        self.directory.join(format!("{}.evdata", key.digest))
    }

    /// Writes and compresses data into the store for the given key.
    pub fn set(&self, key: Id, value: &[u8]) -> Result<(), StoreError> {
        let file_path = self.file_path(key);

        let mut temp_file = NamedTempFile::new_in(&self.directory)?;

        let compressed_value = zstd::encode_all(Cursor::new(value), 0)?;
        temp_file.write_all(&compressed_value)?;

        debug!("Writing to store: {:?}", file_path);

        temp_file.persist(&file_path)?;

        Ok(())
    }

    /// Reads and decompresses data from the store for the given key.
    /// Returns `None` if the key does not exist.
    pub fn get(&self, key: Id) -> Result<Option<Vec<u8>>, StoreError> {
        let file_path = self.file_path(key);
        debug!("Reading from store: {:?}", file_path);
        match fs::read(&file_path) {
            Ok(data) => {
                let decompressed_data = zstd::decode_all(Cursor::new(data))?;
                Ok(Some(decompressed_data))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!("File not found in store: {:?}", file_path);
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Removes a key from the store. Succeeds silently if the key did not exist.
    pub fn remove(&self, key: Id) -> Result<(), StoreError> {
        let file_path = self.file_path(key);
        debug!("Removing from store: {:?}", file_path);
        match fs::remove_file(&file_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Iterates over the store directory to collect all valid keys currently stored.
    pub fn keys(&self) -> Result<Vec<Id>, StoreError> {
        WalkDir::new(&self.directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "evdata"))
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
            })
            .map(|stem| {
                stem.parse::<u64>()
                    .map(|digest| Id { digest })
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e).into())
            })
            .collect()
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
        let store = FileStore::new(&store_dir).unwrap();

        assert!(store_dir.exists());
        assert_eq!(store.directory(), store_dir);
    }

    #[test]
    fn test_directory() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path()).unwrap();

        assert_eq!(store.directory(), dir.path());
    }

    #[test]
    fn test_file_path() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };

        let expected = dir.path().join("12345.evdata");
        assert_eq!(store.file_path(id), expected);
    }

    #[test]
    fn test_set() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };
        let data = b"test data";

        store.set(id, data).unwrap();

        let file_path = store.file_path(id);
        let read_data = fs::read(file_path).unwrap();

        let expected_compressed_data = zstd::encode_all(Cursor::new(data), 0).unwrap();
        assert_eq!(read_data, expected_compressed_data);
    }

    #[test]
    fn test_get() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };
        let data = b"test data";

        let compressed_data = zstd::encode_all(Cursor::new(data), 0).unwrap();
        fs::write(store.file_path(id), compressed_data).unwrap();

        let read_data = store.get(id).unwrap().unwrap();
        assert_eq!(read_data, data);

        let missing_id = Id { digest: 54321 };
        let missing_data = store.get(missing_id).unwrap();
        assert!(missing_data.is_none());
    }

    #[test]
    fn test_remove() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path()).unwrap();
        let id = Id { digest: 12345 };
        let data = b"test data";

        store.set(id, data).unwrap();
        assert!(store.get(id).unwrap().is_some());

        store.remove(id).unwrap();
        assert!(store.get(id).unwrap().is_none());

        store.remove(id).unwrap();
    }

    #[test]
    fn test_keys() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path()).unwrap();

        let id1 = Id { digest: 12345 };
        let id2 = Id { digest: 67890 };

        store.set(id1, b"data1").unwrap();
        store.set(id2, b"data2").unwrap();

        let mut keys = store.keys().unwrap();
        keys.sort_by_key(|id| id.digest);

        assert_eq!(keys, vec![id1, id2]);
    }
}
