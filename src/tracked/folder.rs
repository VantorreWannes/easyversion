use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{file::TrackedFile, TrackedItem};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFolder {
    folder_path: PathBuf,
    patch_dir: PathBuf,
    items: Vec<TrackedItem>,
}

impl TrackedFolder {
    pub fn new(folder_path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
        let mut items = vec![];
        for entry in folder_path.as_ref().read_dir()? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let tracked_folder = TrackedFolder::new(entry.path(), patch_dir.as_ref())?;
                items.push(tracked_folder.into());
            } else {
                let tracked_file = TrackedFile::new(entry.path(), patch_dir.as_ref());
                items.push(tracked_file.into());
            }
        }
        Ok(Self {
            folder_path: folder_path.as_ref().to_path_buf(),
            patch_dir: patch_dir.as_ref().to_path_buf(),
            items,
        })
    }

    pub fn save(&mut self) -> io::Result<()> {
        for item in self.items.iter_mut() {
            item.save()?;
        }
        Ok(())
    }

    pub fn load(&mut self, index: usize) -> io::Result<()> {
        for item in self.items.iter_mut() {
            item.load(index)?;
        }
        Ok(())
    }

    pub fn delete(&mut self, index: usize) -> io::Result<()> {
        for item in self.items.iter_mut() {
            item.delete(index)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tracked_folder_tests {
    use super::*;

    #[test]
    fn test_new() -> io::Result<()> {
        let tracked_folder = TrackedFolder::new("test-data/tracked/folder/items", "test-data/tracked/folder/patches")?;
        assert_eq!(tracked_folder.items.len(), 2);
        Ok(())
    }

    #[test]
    fn test_save() -> io::Result<()> {
        let mut tracked_folder = TrackedFolder::new("test-data/tracked/folder/items", "test-data/tracked/folder/patches")?;
        tracked_folder.save()
    }

    #[test]
    fn test_load() -> io::Result<()> {
        let mut tracked_folder = TrackedFolder::new("test-data/tracked/folder/items", "test-data/tracked/folder/patches")?;
        tracked_folder.save()?;
        tracked_folder.load(0)?;
        Ok(())
    }
}
