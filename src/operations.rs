use std::{
    collections::HashMap,
    fs,
    hash::Hasher,
    path::{Path, PathBuf},
};

use gxhash::GxHasher;
use rayon::prelude::*;
use thiserror::Error;
use walkdir::WalkDir;

use crate::{
    model::{Id, Manifest, Snapshot},
    store::{KVStore, StoreError},
};

#[derive(Debug, Error)]
pub enum OperationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Store(#[from] StoreError),
}

fn data_id(data: &[u8]) -> Id {
    let mut hasher = GxHasher::default();
    hasher.write(data);
    Id {
        digest: hasher.finish(),
    }
}

fn path_id(path: &Path) -> Id {
    let mut hasher = GxHasher::default();
    hasher.write(path.to_string_lossy().as_bytes());
    Id {
        digest: hasher.finish(),
    }
}

fn store_file(store: &KVStore, path: &Path) -> Result<(PathBuf, Id), OperationError> {
    let data = fs::read(path)?;
    let key = data_id(&data);
    store.set(key, &data)?;
    Ok((path.to_path_buf(), key))
}

fn manifest(store: &KVStore, directory: &Path) -> Result<Manifest, OperationError> {
    let mut manifest = Manifest {
        files: HashMap::new(),
    };

    let entries: Vec<PathBuf> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    let new_entries: Vec<(PathBuf, Id)> = entries
        .par_iter()
        .map(|path| store_file(store, path))
        .collect::<Result<Vec<_>, OperationError>>()?;

    for (path, id) in new_entries {
        manifest.files.insert(path, id);
    }

    Ok(manifest)
}

fn snapshot(
    store: &KVStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<Snapshot, OperationError> {
    let manifest = manifest(store, directory)?;
    Ok(Snapshot { comment, manifest })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_id() {
        let data = b"test data";
        let mut hasher = GxHasher::default();
        hasher.write(data);
        let expected_digest = hasher.finish();

        let generated_id = data_id(data);
        assert_eq!(generated_id.digest, expected_digest);
    }

    #[test]
    fn test_store_file() {
        let dir = tempdir().unwrap();
        let store_dir = dir.path().join("store");
        let store = KVStore::new(&store_dir).unwrap();

        let file_path = dir.path().join("test_file.txt");
        let data = b"hello world";
        fs::write(&file_path, data).unwrap();

        let (stored_path, stored_id) = store_file(&store, &file_path).unwrap();

        assert_eq!(stored_path, file_path);

        let retrieved_data = store.get(stored_id).unwrap();
        assert_eq!(retrieved_data, data);
    }

    #[test]
    fn test_manifest() {
        let dir = tempdir().unwrap();
        let store_dir = dir.path().join("store");
        let store = KVStore::new(&store_dir).unwrap();

        let target_dir = dir.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();

        let file1_path = target_dir.join("file1.txt");
        let data1 = b"file1 data";
        fs::write(&file1_path, data1).unwrap();

        let file2_path = target_dir.join("file2.txt");
        let data2 = b"file2 data";
        fs::write(&file2_path, data2).unwrap();

        let manifest_result = manifest(&store, &target_dir).unwrap();

        assert_eq!(manifest_result.files.len(), 2);

        let id1 = data_id(data1);
        let id2 = data_id(data2);

        assert_eq!(manifest_result.files.get(&file1_path), Some(&id1));
        assert_eq!(manifest_result.files.get(&file2_path), Some(&id2));
    }
}
