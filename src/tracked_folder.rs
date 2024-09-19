use std::{
    io, path::{Path, PathBuf}
};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::tracked_file::TrackedFile;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFolder {
    path: PathBuf,
    files: Vec<TrackedFile>,
}

impl TrackedFolder {
    pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
        let mut files = vec![];
        for entry in WalkDir::new(&path)
        {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue;
            }
            let tracked_file = TrackedFile::new(entry.path(), &patch_dir);
            files.push(tracked_file);
        }
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            files,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&mut self) -> io::Result<()> {
        for file in self.files.iter_mut() {
            file.save()?;
        }
        Ok(())
    }

    pub fn files(&self) -> &[TrackedFile] {
        &self.files
    }

    pub fn load(&mut self, index: usize) -> io::Result<()> {
        for file in self.files.iter_mut() {
            file.load(index)?;
        }
        Ok(())
    }

    pub fn delete(&mut self, index: usize) -> io::Result<()> {
        for file in self.files.iter_mut() {
            file.delete(index)?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[cfg(test)]
mod tracked_folder_tests {
    use std::fs;

    use super::*;

    fn test_dir(name: &str) -> io::Result<(PathBuf, PathBuf)> {
        let dir = dirs::config_dir()
            .unwrap()
            .join("easyversion")
            .join("tests")
            .join("tracked_folder")
            .join(name);
        fs::create_dir_all(&dir)?;
        let patch_dir = dir.join("patch");
        let tracked_dir = dir.join("tracked_folder");
        let tracked_dir_subfolder = tracked_dir.join("subfolder");
        fs::create_dir_all(&patch_dir)?;
        fs::create_dir_all(&tracked_dir)?;
        fs::create_dir_all(&tracked_dir_subfolder)?;
        fs::write(tracked_dir.join("file.txt"), b"123")?;
        fs::write(tracked_dir_subfolder.join("file.txt"), b"123")?;
        Ok((tracked_dir, patch_dir))
    }

    #[test]
    fn new() -> io::Result<()> {
        let (tracked_dir, patch_dir) = test_dir("new")?;
        let tracked_folder = TrackedFolder::new(&tracked_dir, &patch_dir);
        assert!(tracked_folder.is_ok());
        Ok(())
    }

    #[test]
    fn save() -> io::Result<()> {
        let (tracked_dir, patch_dir) = test_dir("save")?;
        let mut tracked_folder = TrackedFolder::new(&tracked_dir, &patch_dir)?;
        tracked_folder.save()?;
        Ok(())
    }

    #[test]
    fn load() -> io::Result<()> {
        let (tracked_dir, patch_dir) = test_dir("load")?;
        let mut tracked_folder = TrackedFolder::new(&tracked_dir, &patch_dir)?;
        tracked_folder.save()?;
        tracked_folder.load(0)?;
        Ok(())
    }

    #[test]
    fn delete() -> io::Result<()> {
        let (tracked_dir, patch_dir) = test_dir("delete")?;
        let mut tracked_folder = TrackedFolder::new(&tracked_dir, &patch_dir)?;
        tracked_folder.save()?;
        tracked_folder.save()?;
        tracked_folder.delete(1)
    }
}
