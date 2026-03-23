use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

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

fn data_id(path: &Path) -> Result<Id, OperationError> {
    let mut hasher = blake3::Hasher::new();
    let mut file = fs::File::open(path)?;

    // Streams file directly into hasher. Memory usage stays < 1MB regardless of file size.
    std::io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();

    // Extract first 8 bytes to fit into existing u64 Id structure
    let mut digest_bytes = [0u8; 8];
    digest_bytes.copy_from_slice(&hash_bytes.as_bytes()[0..8]);
    let digest = u64::from_le_bytes(digest_bytes);

    Ok(Id { digest })
}

fn path_id(path: &Path) -> Id {
    let mut hasher = blake3::Hasher::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let hash_bytes = hasher.finalize();
    let mut digest_bytes = [0u8; 8];
    digest_bytes.copy_from_slice(&hash_bytes.as_bytes()[0..8]);
    Id {
        digest: u64::from_le_bytes(digest_bytes),
    }
}

fn store_file(
    store: &FileStore,
    path: &Path,
    base_dir: &Path,
) -> Result<(PathBuf, Id), OperationError> {
    let id = data_id(path)?;

    let keys = store.keys()?;
    if !keys.contains(&id) {
        let data = fs::read(path)?;
        store.set(id, &data)?;
    }

    let rel_path = path.strip_prefix(base_dir).unwrap_or(path).to_path_buf();

    Ok((rel_path, id))
}

fn manifest(store: &FileStore, directory: &Path) -> Result<Manifest, OperationError> {
    let mut manifest = Manifest {
        files: HashMap::new(),
    };

    let entries: Vec<PathBuf> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !e.file_type().is_symlink())
        .map(|e| e.into_path())
        .collect();

    let new_entries: Vec<(PathBuf, Id)> = entries
        .par_iter()
        .map(|path| store_file(store, path, directory))
        .collect::<Result<Vec<_>, OperationError>>()?;

    for (rel_path, id) in new_entries {
        manifest.files.insert(rel_path, id);
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

pub fn history(
    history_store: &FileStore,
    directory: &Path,
) -> Result<Option<History>, OperationError> {
    let key = path_id(directory);
    match history_store.get(key)? {
        Some(json_data) => Ok(Some(serde_json::from_slice(&json_data)?)),
        None => Ok(None),
    }
}

pub fn save(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<(), OperationError> {
    let snapshot = snapshot(data_store, directory, comment.clone())?;
    let mut hist = history(history_store, directory)?.unwrap_or_default();
    hist.snapshots.push(snapshot);

    history_store.set(path_id(directory), &serde_json::to_vec(&hist)?)?;
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

fn load(
    data_store: &FileStore,
    manifest: &Manifest,
    target_directory: &Path,
) -> Result<(), OperationError> {
    for (rel_path, id) in &manifest.files {
        let dest_path = target_directory.join(rel_path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match data_store.get(*id)? {
            Some(data) => fs::write(dest_path, data)?,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Referenced data blob missing",
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
    let mut hist = history(history_store, source_directory)?.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Source history not found")
    })?;

    let target_index = match version {
        Version::Latest => hist.snapshots.len().saturating_sub(1),
        Version::Specific(idx) => idx,
    };

    hist.snapshots.truncate(target_index + 1);

    history_store.set(path_id(target_directory), &serde_json::to_vec(&hist)?)?;

    let target_manifest = &hist.snapshots[target_index].manifest;
    if target_directory.exists() {
        let entries = walkdir::WalkDir::new(target_directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !e.file_type().is_symlink())
            .map(|e| e.into_path());

        for path in entries {
            let rel_path = path
                .strip_prefix(target_directory)
                .unwrap_or(&path)
                .to_path_buf();
            if !target_manifest.files.contains_key(&rel_path) {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    load(data_store, target_manifest, target_directory)?;
    Ok(())
}

pub fn clean(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
) -> Result<(), OperationError> {
    history_store.remove(path_id(directory))?;

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

    for key in data_store.keys()? {
        if !used_ids.contains(&key) {
            data_store.remove(key)?;
        }
    }
    Ok(())
}
