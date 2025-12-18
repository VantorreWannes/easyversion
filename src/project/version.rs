use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::Context;
use gxhash::HashMap;
use log;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::file::id::FileId;
use crate::file::storage::FileStorage;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
pub struct Version {
    comment: Option<String>,
    files: HashMap<PathBuf, FileId>,
}

impl Version {
    pub fn new(comment: Option<&str>) -> Self {
        Self {
            comment: comment.map(ToString::to_string),
            files: Default::default(),
        }
    }

    pub fn comment(&self) -> &Option<String> {
        &self.comment
    }

    #[allow(dead_code)]
    pub fn files(&self) -> &HashMap<PathBuf, FileId> {
        &self.files
    }

    fn compute_file_entry(
        storage: &FileStorage,
        source_directory_path: &Path,
        file_path: &Path,
    ) -> anyhow::Result<(PathBuf, FileId)> {
        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;
        let mut reader = BufReader::with_capacity(crate::BUFFER_SIZE, file);
        let file_id = storage
            .store(&mut reader)
            .with_context(|| format!("Failed to store file content: {:?}", file_path))?;
        let relative_path = file_path
            .strip_prefix(source_directory_path)
            .with_context(|| {
                format!(
                    "Failed to strip prefix {:?} from {:?}",
                    source_directory_path, file_path
                )
            })?
            .to_path_buf();
        Ok((relative_path, file_id))
    }

    #[allow(dead_code)]
    pub fn store_file(
        &mut self,
        storage: &FileStorage,
        source_directory_path: &Path,
        file_path: &Path,
    ) -> anyhow::Result<()> {
        let (relative_path, file_id) =
            Self::compute_file_entry(storage, source_directory_path, file_path)?;

        self.files.insert(relative_path, file_id);
        Ok(())
    }

    pub fn store_directory(
        &mut self,
        storage: &FileStorage,
        directory_path: &Path,
    ) -> anyhow::Result<()> {
        log::debug!("Storing directory (parallel): {:?}", directory_path);

        let entries: Vec<PathBuf> = WalkDir::new(directory_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();

        // Calculate file IDs in parallel
        let new_entries: Vec<(PathBuf, FileId)> = entries
            .par_iter()
            .map(|path| Self::compute_file_entry(storage, directory_path, path))
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Batch insert
        for (path, id) in new_entries {
            self.files.insert(path, id);
        }

        log::debug!(
            "Stored {} files from directory: {:?}",
            entries.len(),
            directory_path
        );
        Ok(())
    }

    pub fn restore_file_to(
        &self,
        storage: &FileStorage,
        target_directory_path: &Path,
        file_path: &Path,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        let full_file_path = target_directory_path.join(file_path);
        if let Some(parent) = full_file_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directory for {:?}",
                    &full_file_path
                )
            })?;
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&full_file_path)
            .with_context(|| format!("Failed to open target file {:?}", &full_file_path))?;

        let mut buf_writer = BufWriter::with_capacity(crate::BUFFER_SIZE, file);
        storage
            .retrieve(file_id, &mut buf_writer)
            .with_context(|| format!("Failed to retrieve data to {:?}", &full_file_path))?;
        Ok(())
    }

    pub fn restore_directory_to(
        &self,
        storage: &FileStorage,
        target_directory_path: &Path,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Restoring version to directory (parallel): {:?}",
            target_directory_path
        );
        if !target_directory_path.try_exists()? {
            log::trace!("Creating target directory: {:?}", target_directory_path);
            fs::create_dir_all(target_directory_path)?;
        }

        let count = self.files.len();

        self.files
            .par_iter()
            .try_for_each(|(relative_path, &file_id)| {
                self.restore_file_to(storage, target_directory_path, relative_path, file_id)
            })?;

        log::debug!(
            "Restored {} files to directory: {:?}",
            count,
            target_directory_path
        );
        Ok(())
    }

    pub fn clean_up(&self, storage: &FileStorage) -> anyhow::Result<()> {
        let count = self.files.len();
        log::debug!("Cleaning up version with {} files", count);
        self.files
            .par_iter()
            .try_for_each(|(_, &file_id)| storage.decrement_references(file_id))?;
        log::debug!("Version cleanup completed");
        Ok(())
    }

    pub fn increment_references(&self, storage: &FileStorage) -> anyhow::Result<Self> {
        let count = self.files.len();
        log::debug!("Incrementing references for {} files in version", count);
        self.files
            .par_iter()
            .try_for_each(|(_, &file_id)| storage.increment_references(file_id))?;
        log::debug!("Version references incremented successfully");
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    use super::*;

    const EMPTY_DATA: [u8; 0] = [];
    const SMALL_DATA: [u8; 1] = [1];

    #[test]
    fn test_store_file_adds_to_files_map() {
        let temp_storage_dir = TempDir::new().unwrap();
        // storage is now mostly used immutably
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_version_dir = TempDir::new().unwrap();
        let test_file_path = temp_version_dir.path().join("test_file.txt");
        fs::write(&test_file_path, EMPTY_DATA).unwrap();

        version
            .store_file(&storage, temp_version_dir.path(), &test_file_path)
            .unwrap();

        let relative_path = PathBuf::from("test_file.txt");
        assert!(version.files.contains_key(&relative_path));
    }

    #[test]
    fn test_store_file_fails_on_directory() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_dir = TempDir::new().unwrap();
        let directory_path = temp_dir.path().join("a_directory");
        fs::create_dir(&directory_path).unwrap();

        let result = version.store_file(&storage, temp_dir.path(), &directory_path);

        assert!(result.is_err());
        assert!(version.files.is_empty());
    }

    #[test]
    fn test_store_directory_processes_all_files() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");
        let subdirectory = temp_dir.path().join("sub");
        fs::create_dir(&subdirectory).unwrap();
        let file2_path = subdirectory.join("file2.txt");
        fs::write(&file1_path, EMPTY_DATA).unwrap();
        fs::write(&file2_path, SMALL_DATA).unwrap();

        version.store_directory(&storage, temp_dir.path()).unwrap();

        assert_eq!(version.files.len(), 2);
        assert!(version.files.contains_key(&PathBuf::from("file1.txt")));
        assert!(version.files.contains_key(&PathBuf::from("sub/file2.txt")));
    }

    #[test]
    fn test_store_directory_handles_empty_directory() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_dir = TempDir::new().unwrap();

        version.store_directory(&storage, temp_dir.path()).unwrap();

        assert!(version.files.is_empty());
    }

    #[test]
    fn test_restore_file_to_creates_file_and_directories() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_version_dir = TempDir::new().unwrap();
        let test_file_path = temp_version_dir.path().join("test_file.txt");
        fs::write(&test_file_path, SMALL_DATA).unwrap();
        version
            .store_file(&storage, temp_version_dir.path(), &test_file_path)
            .unwrap();
        let temp_restore_dir = TempDir::new().unwrap();

        let file_id = *version.files.get(&PathBuf::from("test_file.txt")).unwrap();

        version
            .restore_file_to(
                &storage,
                temp_restore_dir.path(),
                &PathBuf::from("test_file.txt"),
                file_id,
            )
            .unwrap();

        let restored_file = temp_restore_dir.path().join("test_file.txt");
        assert!(restored_file.exists());
        assert_eq!(fs::read(&restored_file).unwrap(), SMALL_DATA);
    }

    #[test]
    fn test_restore_directory_to_creates_directories_and_files() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_dir = TempDir::new().unwrap();
        let subdirectory = temp_dir.path().join("sub");
        fs::create_dir(&subdirectory).unwrap();
        let file_path = subdirectory.join("nested_file.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();
        version.store_directory(&storage, temp_dir.path()).unwrap();
        let temp_restore_dir = TempDir::new().unwrap();

        version
            .restore_directory_to(&storage, temp_restore_dir.path())
            .unwrap();

        let restored_file = temp_restore_dir.path().join("sub/nested_file.txt");
        assert!(restored_file.exists());
        assert!(temp_restore_dir.path().join("sub").is_dir());
        assert_eq!(fs::read(&restored_file).unwrap(), SMALL_DATA);
    }

    #[test]
    fn test_clean_up_decrements_references() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_version_dir = TempDir::new().unwrap();
        let test_file_path = temp_version_dir.path().join("test_file.txt");
        fs::write(&test_file_path, SMALL_DATA).unwrap();
        version
            .store_file(&storage, temp_version_dir.path(), &test_file_path)
            .unwrap();
        let file_id = version.files.values().next().copied().unwrap();

        version.clean_up(&storage).unwrap();

        assert!(!storage.contains_references(file_id));
    }

    #[test]
    fn test_increment_references_increases_counts() {
        let temp_storage_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let mut version = Version::default();
        let temp_version_dir = TempDir::new().unwrap();
        let test_file_path = temp_version_dir.path().join("test_file.txt");
        fs::write(&test_file_path, SMALL_DATA).unwrap();
        version
            .store_file(&storage, temp_version_dir.path(), &test_file_path)
            .unwrap();
        let file_id = *version.files.values().next().unwrap();
        let initial_count = storage.reference_count(file_id).unwrap();

        let new_version = version.increment_references(&storage).unwrap();

        assert_eq!(storage.reference_count(file_id).unwrap(), initial_count + 1);

        assert_eq!(version, new_version);
    }
}
