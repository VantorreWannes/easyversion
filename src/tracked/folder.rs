use std::{error::Error, fmt::Display, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use super::{
    file::{TrackedFile, TrackedFileError},
    TrackedItem, Version,
};

#[derive(Debug)]
pub enum TrackedFolderError {
    FolderDoesntExist,
    TrackedFileError(TrackedFileError),
    ReadFolderError(io::Error),
}

impl Display for TrackedFolderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackedFolderError::FolderDoesntExist => write!(f, "Folder doesn't exist"),
            TrackedFolderError::TrackedFileError(tracked_file_error) => tracked_file_error.fmt(f),
            TrackedFolderError::ReadFolderError(error) => error.fmt(f),
        }
    }
}

impl Error for TrackedFolderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TrackedFolderError::FolderDoesntExist => None,
            TrackedFolderError::TrackedFileError(tracked_file_error) => Some(tracked_file_error),
            TrackedFolderError::ReadFolderError(error) => Some(error),
        }
    }
}

impl From<TrackedFileError> for TrackedFolderError {
    fn from(err: TrackedFileError) -> Self {
        Self::TrackedFileError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct TrackedFolder {
    tracked_items: Vec<TrackedItem>,
    version_count: usize,
}

impl TrackedFolder {
    pub fn new(
        folder_path: impl AsRef<Path>,
        patch_dir: impl AsRef<Path>,
    ) -> Result<Self, TrackedFolderError> {
        let folder_path = folder_path.as_ref();
        let patch_dir = patch_dir.as_ref();
        if !folder_path.exists() {
            return Err(TrackedFolderError::FolderDoesntExist);
        }
        let mut tracked_items: Vec<TrackedItem> = vec![];
        for entry in fs::read_dir(folder_path).map_err(TrackedFolderError::ReadFolderError)? {
            let entry = entry.map_err(TrackedFolderError::ReadFolderError)?;
            let path = entry.path();
            if path.is_dir() {
                tracked_items.push(TrackedFolder::new(path, patch_dir)?.into());
            } else if path.is_file() {
                tracked_items.push(TrackedFile::new(path, patch_dir)?.into());
            }
        }
        Ok(Self {
            tracked_items,
            version_count: 0,
        })
    }

    pub fn items(&self) -> &[TrackedItem] {
        &self.tracked_items
    }
}

impl Version for TrackedFolder {
    fn commit(&mut self) -> Result<(), super::VersionError> {
        for tracked_item in self.tracked_items.iter_mut() {
            tracked_item.commit()?;
        }
        self.version_count += 1;
        Ok(())
    }

    fn load_version(&self, index: usize) -> Result<(), super::VersionError> {
        for tracked_item in self.tracked_items.iter() {
            tracked_item.load_version(index)?;
        }
        Ok(())
    }

    fn delete_version(&mut self, index: usize) -> Result<(), super::VersionError> {
        for tracked_item in self.tracked_items.iter_mut() {
            tracked_item.delete_version(index)?;
        }
        self.version_count = index;
        Ok(())
    }

    fn version_count(&self) -> usize {
        self.version_count
    }
}

#[cfg(test)]
mod tracked_folder_tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn new() {
        let dir = TempDir::new("easyversion").unwrap();
        let folder_path = dir.path().join("folder");
        fs::create_dir(&folder_path).unwrap();
        let tracked_folder = TrackedFolder::new(&folder_path, dir.path()).unwrap();
        assert_eq!(tracked_folder.version_count(), 0);
    }

    #[test]
    fn commit() {
        let dir = TempDir::new("easyversion").unwrap();
        let folder_path = dir.path().join("folder");
        fs::create_dir(&folder_path).unwrap();
        let mut tracked_folder = TrackedFolder::new(&folder_path, dir.path()).unwrap();
        tracked_folder.commit().unwrap();
        assert_eq!(tracked_folder.version_count(), 1);
    }

    #[test]
    fn load_version() {
        let dir = TempDir::new("easyversion").unwrap();
        let folder_path = dir.path().join("folder");
        fs::create_dir(&folder_path).unwrap();
        let mut tracked_folder = TrackedFolder::new(&folder_path, dir.path()).unwrap();
        tracked_folder.commit().unwrap();
        tracked_folder.load_version(0).unwrap();
        assert_eq!(tracked_folder.version_count(), 1);
    }

    #[test]
    fn delete_version() {
        let dir = TempDir::new("easyversion").unwrap();
        let folder_path = dir.path().join("folder");
        fs::create_dir(&folder_path).unwrap();
        let mut tracked_folder = TrackedFolder::new(&folder_path, dir.path()).unwrap();
        tracked_folder.commit().unwrap();
        tracked_folder.delete_version(0).unwrap();
        assert_eq!(tracked_folder.version_count(), 0);
    }
}
