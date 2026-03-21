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
    model::{History, Id, Manifest, Snapshot},
    store::{FileStore, StoreError},
};

#[derive(Debug, Error)]
pub enum OperationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Store(#[from] StoreError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
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

fn store_file(store: &FileStore, path: &Path) -> Result<(PathBuf, Id), OperationError> {
    let data = fs::read(path)?;
    let key = data_id(&data);
    store.set(key, &data)?;
    Ok((path.to_path_buf(), key))
}

fn manifest(store: &FileStore, directory: &Path) -> Result<Manifest, OperationError> {
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
    store: &FileStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<Snapshot, OperationError> {
    let manifest = manifest(store, directory)?;
    Ok(Snapshot { comment, manifest })
}

pub fn save(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<(), OperationError> {
    let snapshot = snapshot(data_store, directory, comment)?;

    let key = path_id(directory);

    let mut history: History = match history_store.get(key)? {
        Some(json_data) => serde_json::from_slice(&json_data)?,
        None => History::default(),
    };

    history.snapshots.push(snapshot);

    let serialized_history = serde_json::to_vec(&history)?;
    history_store.set(key, &serialized_history)?;

    Ok(())
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
        let store = FileStore::new(&store_dir).unwrap();

        let file_path = dir.path().join("test_file.txt");
        let data = b"hello world";
        fs::write(&file_path, data).unwrap();

        let (stored_path, stored_id) = store_file(&store, &file_path).unwrap();

        assert_eq!(stored_path, file_path);

        let retrieved_data = store.get(stored_id).unwrap().unwrap();
        assert_eq!(retrieved_data, data);
    }

    #[test]
    fn test_manifest() {
        let dir = tempdir().unwrap();
        let store_dir = dir.path().join("store");
        let store = FileStore::new(&store_dir).unwrap();

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

    #[test]
    fn test_path_id() {
        let path = Path::new("/test/path");
        let mut hasher = GxHasher::default();
        hasher.write(path.to_string_lossy().as_bytes());
        let expected_digest = hasher.finish();

        let generated_id = path_id(path);
        assert_eq!(generated_id.digest, expected_digest);
    }

    #[test]
    fn test_snapshot() {
        let dir = tempdir().unwrap();
        let store_dir = dir.path().join("store");
        let store = FileStore::new(&store_dir).unwrap();

        let target_dir = dir.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();
        let file_path = target_dir.join("file1.txt");
        fs::write(&file_path, b"data").unwrap();

        let comment = Some("First snapshot".to_string());
        let snap = snapshot(&store, &target_dir, comment.clone()).unwrap();

        assert_eq!(snap.comment, comment);
        assert_eq!(snap.manifest.files.len(), 1);
        assert!(snap.manifest.files.contains_key(&file_path));
    }

    #[test]
    fn test_save() {
        let dir = tempdir().unwrap();
        let data_store_dir = dir.path().join("data_store");
        let history_store_dir = dir.path().join("history_store");

        let data_store = FileStore::new(&data_store_dir).unwrap();
        let history_store = FileStore::new(&history_store_dir).unwrap();

        let target_dir = dir.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();
        let file_path = target_dir.join("file1.txt");
        fs::write(&file_path, b"data").unwrap();

        let comment = Some("First commit".to_string());
        save(&data_store, &history_store, &target_dir, comment.clone()).unwrap();

        let p_id = path_id(&target_dir);
        let history_data = history_store.get(p_id).unwrap().unwrap();
        let history: History = serde_json::from_slice(&history_data).unwrap();

        assert_eq!(history.snapshots.len(), 1);
        assert_eq!(history.snapshots[0].comment, comment);
        assert_eq!(history.snapshots[0].manifest.files.len(), 1);
        assert!(history.snapshots[0].manifest.files.contains_key(&file_path));
    }
}
