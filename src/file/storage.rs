use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Context;
use gxhash::{GxHasher, HashMap};
use log;
use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::config::Config;
use crate::file::HashingReader;
use crate::file::id::FileId;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStorage {
    directory: PathBuf,
    references: Mutex<HashMap<FileId, usize>>,
}

impl FileStorage {
    pub fn new(directory: &Path) -> anyhow::Result<Self> {
        log::debug!("Creating new FileStorage in directory: {:?}", directory);
        fs::create_dir_all(directory)
            .with_context(|| format!("Failed to create storage directory at {:?}", directory))?;
        log::debug!("FileStorage created successfully in: {:?}", directory);
        Ok(Self {
            directory: directory.to_path_buf(),
            references: Mutex::new(HashMap::default()),
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn file_path(&self, file_id: FileId) -> PathBuf {
        self.directory.join(format!("{}.lz4.ezfile", file_id))
    }

    fn compute_and_write<R: Read>(
        reader: &mut R,
        directory: &Path,
    ) -> anyhow::Result<(FileId, NamedTempFile)> {
        let temp_file = NamedTempFile::new_in(directory)
            .context("Failed to create temporary file for storage")?;

        let mut hashing_reader = HashingReader::new(reader, GxHasher::default());
        let mut writer = FrameEncoder::new(temp_file);

        io::copy(&mut hashing_reader, &mut writer)
            .context("Failed to compress and write data to temporary file")?;
        let temp_file = writer.finish().context("Failed to finalize LZ4 frame")?;

        let hash = hashing_reader.finalize();
        Ok((FileId::new(hash), temp_file))
    }

    fn persist_entry(&self, file_id: FileId, temp_file: NamedTempFile) -> anyhow::Result<()> {
        {
            let mut refs = self
                .references
                .lock()
                .map_err(|_| anyhow::anyhow!("Storage mutex poisoned"))?;
            if let Some(count) = refs.get_mut(&file_id) {
                *count += 1;
                log::debug!(
                    "File {} already exists. Reference count incremented to {}",
                    file_id,
                    count
                );
                return Ok(());
            }
        }

        let target = self.file_path(file_id);
        temp_file
            .persist(&target)
            .context(format!("Failed to persist file to {:?}", target))?;

        let mut refs = self
            .references
            .lock()
            .map_err(|_| anyhow::anyhow!("Storage mutex poisoned"))?;

        if let Some(count) = refs.get_mut(&file_id) {
            *count += 1;
        } else {
            refs.insert(file_id, 1);
            log::debug!("Persisted new file {}. References: 1", file_id);
        }
        Ok(())
    }

    pub fn increment_references(&self, file_id: FileId) -> anyhow::Result<()> {
        log::debug!("Incrementing references for file ID: {}", file_id);
        let mut refs = self
            .references
            .lock()
            .map_err(|_| anyhow::anyhow!("Storage mutex poisoned"))?;

        if let Some(count) = refs.get_mut(&file_id) {
            *count += 1;
            log::debug!("Reference count for {} incremented to {}", file_id, count);
            Ok(())
        } else {
            log::error!(
                "Attempted to increment references for non-existent file ID: {}",
                file_id
            );
            anyhow::bail!("File with id {} was not found", file_id);
        }
    }

    pub fn store<R: Read>(&self, reader: &mut R) -> anyhow::Result<FileId> {
        log::debug!("Storing file in storage directory: {:?}", self.directory);
        let (file_id, temp_file) = Self::compute_and_write(reader, &self.directory)?;
        log::debug!("Computed file ID: {}", file_id);

        self.persist_entry(file_id, temp_file)?;

        log::trace!("File stored successfully with ID: {}", file_id);
        Ok(file_id)
    }

    pub fn retrieve<W: Write>(&self, file_id: FileId, writer: &mut W) -> anyhow::Result<()> {
        log::debug!("Retrieving file with ID: {}", file_id);
        let file_path = self.file_path(file_id);

        log::debug!("Opening file: {:?}", file_path);
        let file = File::open(&file_path)
            .with_context(|| format!("Failed to open storage file: {:?}", file_path))?;

        let mut decoder = FrameDecoder::new(file);
        io::copy(&mut decoder, writer)
            .with_context(|| format!("Failed to decode/write content for file {}", file_id))?;

        log::debug!("File retrieved successfully: {}", file_id);
        Ok(())
    }

    pub fn decrement_references(&self, file_id: FileId) -> anyhow::Result<()> {
        log::debug!("Decrementing references for file ID: {}", file_id);
        let mut refs = self
            .references
            .lock()
            .map_err(|_| anyhow::anyhow!("Storage mutex poisoned"))?;

        if let Some(count) = refs.get_mut(&file_id) {
            if *count > 0 {
                *count -= 1;
                log::debug!("Reference count for {} decremented to {}", file_id, count);
                if *count == 0 {
                    refs.remove(&file_id);
                    let file_path = self.file_path(file_id);
                    log::debug!("Removing file: {:?}", file_path);
                    fs::remove_file(&file_path).with_context(|| {
                        format!("Failed to delete file from storage: {:?}", file_path)
                    })?;
                    log::debug!("File {} removed from storage (no more references)", file_id);
                }
            }
            Ok(())
        } else {
            log::error!(
                "Attempted to decrement references for non-existent file ID: {}",
                file_id
            );
            anyhow::bail!("File with id {} was not found", file_id);
        }
    }

    #[cfg(test)]
    pub fn contains_references(&self, file_id: FileId) -> bool {
        let refs = self.references.lock().unwrap();
        refs.contains_key(&file_id)
    }

    #[cfg(test)]
    pub fn reference_count(&self, file_id: FileId) -> Option<usize> {
        let refs = self.references.lock().unwrap();
        refs.get(&file_id).copied()
    }
}

impl Hash for FileStorage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.directory.hash(state);
    }
}

impl Clone for FileStorage {
    fn clone(&self) -> Self {
        let refs = self.references.lock().unwrap().clone();
        Self {
            directory: self.directory.clone(),
            references: Mutex::new(refs),
        }
    }
}

impl PartialEq for FileStorage {
    fn eq(&self, other: &Self) -> bool {
        if self.directory != other.directory {
            return false;
        }
        let self_refs = self.references.lock().unwrap();
        let other_refs = other.references.lock().unwrap();
        *self_refs == *other_refs
    }
}

impl Eq for FileStorage {}

impl Config for FileStorage {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    const EMPTY_DATA: [u8; 0] = [];
    const EMPTY_DATA_HASH: u64 = 17118817743232439212;
    const SMALL_DATA: [u8; 1] = [42];
    #[allow(dead_code)]
    const SMALL_DATA_HASH: u64 = 4843356406056171753;

    #[test]
    fn test_new_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("subdir");

        let _storage = FileStorage::new(&target_path).unwrap();

        assert!(target_path.is_dir());
    }

    #[test]
    fn test_file_path_formats_correctly() {
        let storage = FileStorage::new(Path::new("/tmp/test")).unwrap();
        let file_id = FileId::new(EMPTY_DATA_HASH);

        let path = storage.file_path(file_id);

        assert_eq!(
            path.to_str().unwrap(),
            format!("/tmp/test/{}.lz4.ezfile", file_id)
        );
    }

    #[test]
    fn test_compute_and_write_handles_empty_data() {
        let temp_dir = TempDir::new().unwrap();
        let mut reader = Cursor::new(EMPTY_DATA);

        let (computed_id, _temp_file) =
            FileStorage::compute_and_write(&mut reader, temp_dir.path()).unwrap();

        assert_eq!(computed_id.value(), EMPTY_DATA_HASH);
    }

    #[test]
    fn test_persist_entry_increments_existing_count() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let file_id = FileId::new(EMPTY_DATA_HASH);

        {
            let mut refs = storage.references.lock().unwrap();
            refs.insert(file_id, 5);
        }

        let temp_file = NamedTempFile::new_in(temp_dir.path()).unwrap();
        storage.persist_entry(file_id, temp_file).unwrap();

        assert_eq!(storage.reference_count(file_id).unwrap(), 6);
    }

    #[test]
    fn test_persist_entry_creates_new_entry() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let file_id = FileId::new(EMPTY_DATA_HASH);
        let temp_file = NamedTempFile::new_in(temp_dir.path()).unwrap();

        storage.persist_entry(file_id, temp_file).unwrap();

        assert_eq!(storage.reference_count(file_id).unwrap(), 1);
    }

    #[test]
    fn test_store_generates_id_and_stores() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let mut reader = Cursor::new(EMPTY_DATA);

        let file_id = storage.store(&mut reader).unwrap();

        assert_eq!(file_id.value(), EMPTY_DATA_HASH);
        assert!(storage.contains_references(file_id));
    }

    #[test]
    fn test_increment_references_increments_existing_count() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let mut reader = Cursor::new(EMPTY_DATA);
        let file_id = storage.store(&mut reader).unwrap();

        storage.increment_references(file_id).unwrap();

        assert_eq!(storage.reference_count(file_id).unwrap(), 2);
    }

    #[test]
    fn test_increment_references_fails_on_missing_id() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let missing_id = FileId::new(EMPTY_DATA_HASH);

        let result = storage.increment_references(missing_id);

        assert!(result.is_err());
    }

    #[test]
    fn test_retrieve_gets_stored_data() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let mut reader = Cursor::new(SMALL_DATA);
        let file_id = storage.store(&mut reader).unwrap();
        let mut output = Vec::new();

        storage.retrieve(file_id, &mut output).unwrap();

        assert_eq!(output, SMALL_DATA);
    }

    #[test]
    fn test_retrieve_fails_on_missing_id() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let missing_id = FileId::new(EMPTY_DATA_HASH);
        let mut output = Vec::new();

        let result = storage.retrieve(missing_id, &mut output);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrement_references_removes_to_zero() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let mut reader = Cursor::new(SMALL_DATA);
        let file_id = storage.store(&mut reader).unwrap();

        storage.decrement_references(file_id).unwrap();

        assert!(!storage.contains_references(file_id));
    }

    #[test]
    fn test_decrement_references_fails_on_missing_id() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let missing_id = FileId::new(EMPTY_DATA_HASH);

        let result = storage.decrement_references(missing_id);

        assert!(result.is_err());
    }

    #[test]
    fn test_parallel_store() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path()).unwrap());

        let mut handles = vec![];
        for i in 0..10 {
            let s = Arc::clone(&storage);
            handles.push(thread::spawn(move || {
                let data = [i as u8];
                let mut reader = Cursor::new(data);
                s.store(&mut reader).unwrap()
            }));
        }

        for h in handles {
            let id = h.join().unwrap();
            assert!(storage.contains_references(id));
        }
    }
}
