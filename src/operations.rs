use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hasher,
    path::{Path, PathBuf},
};

use gxhash::GxHasher;
use log::{debug, info};
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
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                log::warn!("Skipping file due to read error: {}", err);
                None
            }
        })
        .filter(|e| e.file_type().is_file() && !e.file_type().is_symlink())
        .map(|e| e.into_path())
        .collect();

    debug!("Found {} files to process", entries.len());

    let new_entries: Vec<(PathBuf, Id)> = entries
        .par_iter()
        .map(|path| store_file(store, path))
        .collect::<Result<Vec<_>, OperationError>>()?;

    for (path, id) in new_entries {
        manifest.files.insert(path, id);
    }

    debug!("Manifest generated with {} files", manifest.files.len());

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

pub fn history(
    history_store: &FileStore,
    directory: &Path,
) -> Result<Option<History>, OperationError> {
    let key = path_id(directory);
    match history_store.get(key)? {
        Some(json_data) => {
            let hist: History = serde_json::from_slice(&json_data)?;
            Ok(Some(hist))
        }
        None => Ok(None),
    }
}

pub fn save(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<(), OperationError> {
    info!("Saving snapshot for {:?}", directory);
    let snapshot = snapshot(data_store, directory, comment.clone())?;

    let mut hist = history(history_store, directory)?.unwrap_or_default();

    hist.snapshots.push(snapshot);
    let snapshot_index = hist.snapshots.len() - 1;
    debug!("Appended snapshot at index {}", snapshot_index);

    let key = path_id(directory);
    let serialized_history = serde_json::to_vec(&hist)?;
    history_store.set(key, &serialized_history)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Version {
    Latest,
    Specific(usize),
}

impl Default for Version {
    fn default() -> Self {
        Version::Latest
    }
}

fn load(data_store: &FileStore, manifest: &Manifest) -> Result<(), OperationError> {
    info!("Loading {} files", manifest.files.len());
    for (dest_path, id) in &manifest.files {
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match data_store.get(*id)? {
            Some(data) => fs::write(dest_path, data)?,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Critical: Referenced data blob {:?} is missing from the store",
                        id.digest
                    ),
                )
                .into());
            }
        }
    }

    Ok(())
}

pub fn split(
    data_store: &FileStore,
    history_store: &FileStore,
    source_directory: &Path,
    target_directory: &Path,
    version: Version,
) -> Result<(), OperationError> {
    info!("Splitting {:?} to {:?}", source_directory, target_directory);
    let mut hist = history(history_store, source_directory)?.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Source history not found")
    })?;

    let target_index = match version {
        Version::Latest => hist.snapshots.len().saturating_sub(1),
        Version::Specific(idx) => idx,
    };

    debug!("Splitting at snapshot index {}", target_index);

    if target_index >= hist.snapshots.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Requested version index out of bounds",
        )
        .into());
    }

    hist.snapshots.truncate(target_index + 1);

    for snapshot in &mut hist.snapshots {
        let mut new_files = HashMap::new();
        for (old_path, id) in snapshot.manifest.files.drain() {
            let relative_path = old_path.strip_prefix(source_directory).unwrap_or(&old_path);
            let new_path = target_directory.join(relative_path);
            new_files.insert(new_path, id);
        }
        snapshot.manifest.files = new_files;
    }

    let target_key = path_id(target_directory);
    let serialized_history = serde_json::to_vec(&hist)?;
    history_store.set(target_key, &serialized_history)?;

    load(data_store, &hist.snapshots[target_index].manifest)?;

    Ok(())
}

pub fn clean(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
) -> Result<(), OperationError> {
    info!("Starting clean operation for {:?}", directory);
    let current_key = path_id(directory);
    history_store.remove(current_key)?;

    let mut used_ids = HashSet::new();

    for key in history_store.keys()? {
        if let Some(json_data) = history_store.get(key)? {
            let hist: History = serde_json::from_slice(&json_data)?;
            for snapshot in hist.snapshots {
                for id in snapshot.manifest.files.values() {
                    used_ids.insert(*id);
                }
            }
        }
    }

    debug!("Found {} referenced data objects", used_ids.len());
    let mut removed_count = 0;

    for key in data_store.keys()? {
        if !used_ids.contains(&key) {
            data_store.remove(key)?;
            removed_count += 1;
        }
    }

    info!(
        "Clean complete: {} unreferenced objects removed",
        removed_count
    );
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
    fn test_history() {
        let dir = tempdir().unwrap();
        let history_store_dir = dir.path().join("history_store");
        let history_store = FileStore::new(&history_store_dir).unwrap();

        let target_dir = dir.path().join("target");

        assert!(history(&history_store, &target_dir).unwrap().is_none());

        let dummy_history = History::default();
        let serialized = serde_json::to_vec(&dummy_history).unwrap();
        history_store
            .set(path_id(&target_dir), &serialized)
            .unwrap();

        let retrieved = history(&history_store, &target_dir).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().snapshots.len(), 0);
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

        let hist = history(&history_store, &target_dir).unwrap().unwrap();

        assert_eq!(hist.snapshots.len(), 1);
        assert_eq!(hist.snapshots[0].comment, comment);
        assert_eq!(hist.snapshots[0].manifest.files.len(), 1);
        assert!(hist.snapshots[0].manifest.files.contains_key(&file_path));
    }

    #[test]
    fn test_load() {
        let dir = tempdir().unwrap();
        let data_store_dir = dir.path().join("data");
        let data_store = FileStore::new(&data_store_dir).unwrap();

        let target_dir = dir.path().join("target");

        let file_data = b"mock file content";
        let file_id = data_id(file_data);
        data_store.set(file_id, file_data).unwrap();

        let mut manifest = Manifest {
            files: HashMap::new(),
        };

        let mock_dest_path = target_dir.join("subfolder/data.txt");
        manifest.files.insert(mock_dest_path.clone(), file_id);

        load(&data_store, &manifest).unwrap();

        assert!(mock_dest_path.exists());
        assert_eq!(fs::read(&mock_dest_path).unwrap(), file_data);
    }

    #[test]
    fn test_split() {
        let dir = tempdir().unwrap();
        let data_store = FileStore::new(&dir.path().join("data")).unwrap();
        let history_store = FileStore::new(&dir.path().join("history")).unwrap();

        let source_dir = dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        let file_path = source_dir.join("hello.txt");
        fs::write(&file_path, b"hello world").unwrap();

        save(&data_store, &history_store, &source_dir, Some("v1".into())).unwrap();

        let target_dir = dir.path().join("target");

        split(
            &data_store,
            &history_store,
            &source_dir,
            &target_dir,
            Version::Latest,
        )
        .unwrap();

        let target_file_path = target_dir.join("hello.txt");
        assert!(target_file_path.exists());
        assert_eq!(fs::read(&target_file_path).unwrap(), b"hello world");

        let target_history = history(&history_store, &target_dir).unwrap().unwrap();

        assert_eq!(target_history.snapshots.len(), 1);
        assert_eq!(target_history.snapshots[0].comment, Some("v1".into()));
    }

    #[test]
    fn test_clean() {
        let dir = tempdir().unwrap();
        let data_store = FileStore::new(&dir.path().join("data")).unwrap();
        let history_store = FileStore::new(&dir.path().join("history")).unwrap();

        let source_dir = dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        let file_path = source_dir.join("referenced.txt");
        let referenced_data = b"i am referenced";
        fs::write(&file_path, referenced_data).unwrap();
        save(&data_store, &history_store, &source_dir, None).unwrap();

        let referenced_id = data_id(referenced_data);

        let unreferenced_data = b"i am ghost";
        let unreferenced_id = data_id(unreferenced_data);
        data_store.set(unreferenced_id, unreferenced_data).unwrap();

        let data_keys = data_store.keys().unwrap();
        assert_eq!(data_keys.len(), 2);
        assert!(data_keys.contains(&referenced_id));
        assert!(data_keys.contains(&unreferenced_id));

        clean(&data_store, &history_store, &source_dir).unwrap();

        assert!(history(&history_store, &source_dir).unwrap().is_none());

        let data_keys_after = data_store.keys().unwrap();
        assert_eq!(data_keys_after.len(), 0);
        assert!(!data_keys_after.contains(&referenced_id));
        assert!(!data_keys_after.contains(&unreferenced_id));

        assert!(data_store.get(referenced_id).unwrap().is_none());
    }
}
