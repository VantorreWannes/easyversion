use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::{file::TrackedFile, Version};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFolder {
    path: PathBuf,
    files: Vec<TrackedFile>,
}

impl TrackedFolder {
    pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&path).into_iter() {
            let entry = entry?;
            if !entry.file_type().is_dir() {
                let tracked_file = TrackedFile::new(entry.path(), &patch_dir)?;
                files.push(tracked_file);
            }
        }

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            files,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn files(&self) -> &[TrackedFile] {
        &self.files
    }
}

impl Version for TrackedFolder {
    fn save(&mut self) -> io::Result<()> {
        for file in self.files.iter_mut() {
            file.save()?;
        }
        Ok(())
    }

    fn load(&mut self, index: usize) -> io::Result<()> {
        for file in self.files.iter_mut() {
            file.load(index)?;
        }
        Ok(())
    }

    fn delete(&mut self, index: usize) -> io::Result<()> {
        for file in self.files.iter_mut() {
            file.delete(index)?;
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.files.first().map(|f| f.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tracked_folder_tests {
    use crate::tracked::version_test_tools::{patch_dir_path, tracked_folder_path};

    use super::*;

    #[test]
    fn new() -> io::Result<()> {
        let dirs = &["tracked_folder", "new"];
        let patch_dir_path = patch_dir_path(dirs)?;
        let tracked_folder_path = tracked_folder_path(dirs)?;
        let tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path);
        assert!(tracked_folder.is_ok());
        Ok(())
    }

    #[test]
    fn save() -> io::Result<()> {
        let dirs = &["tracked_folder", "save"];
        let patch_dir_path = patch_dir_path(dirs)?;
        let tracked_folder_path = tracked_folder_path(dirs)?;
        let mut tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path)?;
        tracked_folder.save()
    }

    #[test]
    fn load() -> io::Result<()> {
        let dirs = &["tracked_folder", "load"];
        let patch_dir_path = patch_dir_path(dirs)?;
        let tracked_folder_path = tracked_folder_path(dirs)?;
        let mut tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path)?;
        tracked_folder.save()?;
        tracked_folder.load(0)
    }

    #[test]
    fn delete() -> io::Result<()> {
        let dirs = &["tracked_folder", "delete"];
        let patch_dir_path = patch_dir_path(dirs)?;
        let tracked_folder_path = tracked_folder_path(dirs)?;
        let mut tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path)?;
        tracked_folder.save()?;
        tracked_folder.delete(0)
    }
}
