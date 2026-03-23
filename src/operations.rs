use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use log::{debug, info, trace};
use rayon::prelude::*;
use thiserror::Error;
use walkdir::WalkDir;

use crate::{
    model::{History, Id, Manifest, Snapshot},
    store::{FileStore, StoreError},
};

/// Represents the various errors that can occur during system operations.
#[derive(Debug, Error)]
pub enum OperationError {
    /// An underlying I/O error occurred.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// An error occurred within the storage layer.
    #[error(transparent)]
    Store(#[from] StoreError),

    /// An error occurred while processing JSON data.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Computes a structural identity (hash) for the physical contents of a file.
fn data_id(path: &Path) -> Result<Id, OperationError> {
    trace!("Computing data ID for file: {:?}", path);
    let mut hasher = blake3::Hasher::new();
    let mut file = fs::File::open(path)?;

    std::io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();

    let mut digest_bytes = [0u8; 8];
    digest_bytes.copy_from_slice(&hash_bytes.as_bytes()[0..8]);
    let digest = u64::from_le_bytes(digest_bytes);

    debug!("Computed data ID {} for file: {:?}", digest, path);
    Ok(Id { digest })
}

/// Computes a structural identity (hash) for a logical file path.
fn path_id(path: &Path) -> Id {
    trace!("Computing path ID for: {:?}", path);
    let mut hasher = blake3::Hasher::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let hash_bytes = hasher.finalize();

    let mut digest_bytes = [0u8; 8];
    digest_bytes.copy_from_slice(&hash_bytes.as_bytes()[0..8]);

    let digest = u64::from_le_bytes(digest_bytes);
    debug!("Computed path ID {} for: {:?}", digest, path);
    Id { digest }
}

/// Stores a file's contents into the data store if its intrinsic identity does not already exist.
/// Returns the file's normalized relative path and its computed identity.
fn store_file(
    store: &FileStore,
    path: &Path,
    base_dir: &Path,
) -> Result<(PathBuf, Id), OperationError> {
    trace!("Evaluating file for storage: {:?}", path);
    let id = data_id(path)?;

    let keys = store.keys()?;
    if !keys.contains(&id) {
        debug!("Blob not found in store, persisting: {}", id.digest);
        let data = fs::read(path)?;
        store.set(id, &data)?;
    } else {
        trace!(
            "Blob already exists in store, skipping persistence: {}",
            id.digest
        );
    }

    let rel_path = path.strip_prefix(base_dir).unwrap_or(path).to_path_buf();

    Ok((rel_path, id))
}

/// Generates a manifest encapsulating the current physical state of a directory boundary.
/// Discovers all files, computes their identities, and ensures their contents are safely stored.
fn manifest(store: &FileStore, directory: &Path) -> Result<Manifest, OperationError> {
    info!("Generating manifest for directory: {:?}", directory);
    let mut manifest = Manifest {
        files: HashMap::new(),
    };

    let entries: Vec<PathBuf> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !e.file_type().is_symlink())
        .map(|e| e.into_path())
        .collect();

    debug!("Found {} files to process for manifest", entries.len());

    let new_entries: Vec<(PathBuf, Id)> = entries
        .par_iter()
        .map(|path| store_file(store, path, directory))
        .collect::<Result<Vec<_>, OperationError>>()?;

    for (rel_path, id) in new_entries {
        manifest.files.insert(rel_path, id);
    }

    info!(
        "Successfully generated manifest with {} entries",
        manifest.files.len()
    );
    Ok(manifest)
}

/// Creates a new immutable snapshot encompassing the directory's current manifest and an optional note.
fn snapshot(
    store: &FileStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<Snapshot, OperationError> {
    info!("Creating snapshot for directory: {:?}", directory);
    let manifest = manifest(store, directory)?;
    debug!("Snapshot created successfully");
    Ok(Snapshot { comment, manifest })
}

/// Retrieves the sequential historical record of snapshots for a given directory context.
pub fn history(
    history_store: &FileStore,
    directory: &Path,
) -> Result<Option<History>, OperationError> {
    info!("Fetching history for directory: {:?}", directory);
    let key = path_id(directory);
    match history_store.get(key)? {
        Some(json_data) => {
            debug!("History found and deserialized successfully");
            Ok(Some(serde_json::from_slice(&json_data)?))
        }
        None => {
            debug!("No history found for directory: {:?}", directory);
            Ok(None)
        }
    }
}

/// Captures the current state of a directory and appends it to its canonical historical record.
pub fn save(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
    comment: Option<String>,
) -> Result<(), OperationError> {
    info!("Saving new snapshot for directory: {:?}", directory);
    let snapshot = snapshot(data_store, directory, comment.clone())?;
    let mut hist = history(history_store, directory)?.unwrap_or_default();
    hist.snapshots.push(snapshot);

    history_store.set(path_id(directory), &serde_json::to_vec(&hist)?)?;
    info!("Successfully appended snapshot to history");
    Ok(())
}

/// Defines the resolution target for extracting a snapshot from history.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Version {
    /// Targets the most recent chronologically appended snapshot.
    Latest,
    /// Targets a specific 0-indexed position within the historical sequence.
    Specific(usize),
}

impl Default for Version {
    /// Defaults to the latest available state.
    fn default() -> Self {
        Version::Latest
    }
}

/// Reconstructs the physical file structures described by a pure manifest into the target directory.
fn load(
    data_store: &FileStore,
    manifest: &Manifest,
    target_directory: &Path,
) -> Result<(), OperationError> {
    info!(
        "Loading manifest state into target directory: {:?}",
        target_directory
    );
    for (rel_path, id) in &manifest.files {
        let dest_path = target_directory.join(rel_path);
        trace!("Restoring file state: {:?}", dest_path);
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
    debug!("Successfully reified manifest state to filesystem");
    Ok(())
}

/// Forks a specific historical state into a target directory context, isolating it as a new boundary.
/// Purges files in the target directory that do not belong to the target state topology.
pub fn split(
    data_store: &FileStore,
    history_store: &FileStore,
    source_directory: &Path,
    target_directory: &Path,
    version: Version,
) -> Result<(), OperationError> {
    info!(
        "Splitting history state from {:?} to {:?}",
        source_directory, target_directory
    );
    let mut hist = history(history_store, source_directory)?.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Source history not found")
    })?;

    let target_index = match version {
        Version::Latest => hist.snapshots.len().saturating_sub(1),
        Version::Specific(idx) => idx,
    };

    debug!("Targeting snapshot timeline index: {}", target_index);
    hist.snapshots.truncate(target_index + 1);

    history_store.set(path_id(target_directory), &serde_json::to_vec(&hist)?)?;

    let target_manifest = &hist.snapshots[target_index].manifest;
    if target_directory.exists() {
        debug!("Pruning extraneous files from target directory to maintain fidelity");
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
                trace!("Deleting file outside of manifest scope: {:?}", path);
                let _ = std::fs::remove_file(path);
            }
        }
    }

    load(data_store, target_manifest, target_directory)?;
    info!("Successfully transitioned target directory to split state");
    Ok(())
}

/// Destroys the historical record for a given directory and executes a global garbage collection
/// routine to reclaim storage from orphaned data blobs lacking any live references.
pub fn clean(
    data_store: &FileStore,
    history_store: &FileStore,
    directory: &Path,
) -> Result<(), OperationError> {
    info!(
        "Commencing aggressive cleanup for directory history: {:?}",
        directory
    );
    history_store.remove(path_id(directory))?;

    let mut used_ids = HashSet::new();
    debug!("Scanning global history state to rebuild reference graph");
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

    debug!(
        "Discovered {} unique globally referenced data blobs",
        used_ids.len()
    );

    for key in data_store.keys()? {
        if !used_ids.contains(&key) {
            trace!("Evicting permanently orphaned data blob: {}", key.digest);
            data_store.remove(key)?;
        }
    }

    info!("Cleanup sequence successfully finalized");
    Ok(())
}
