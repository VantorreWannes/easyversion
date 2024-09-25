use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::{file::TrackedFile, Version, VersionError};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct TrackedFolder {
    path: PathBuf,
    files: Vec<TrackedFile>,
}

impl TrackedFolder {
    pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> Result<Self, VersionError> {
        let mut files = Vec::new();
        for entry in WalkDir::new(&path).into_iter() {
            let entry = entry?;
            if entry.file_type().is_file() {
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
    fn save(&mut self) -> Result<(), VersionError> {
        for file in self.files.iter_mut() {
            file.save()?;
        }
        Ok(())
    }

    fn load(&mut self, index: usize) -> Result<(), VersionError> {
        for file in self.files.iter_mut() {
            file.load(index)?;
        }
        Ok(())
    }

    fn delete(&mut self, index: usize) -> Result<(), VersionError> {
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
    use std::fs;

    use crate::test_tools::dir_path;

    use super::*;

    fn patch_dir(name: &str) -> PathBuf {
        dir_path(&["tracked_folder", "patches", name])
    }

    fn tracked_folder_path(name: &str) -> PathBuf {
        let path = dir_path(&["tracked_folder", "items", name]);
        fs::create_dir_all(&path).expect("Testing shouldn't fail.");
        let item_folder = path.join("folder");
        fs::create_dir_all(&item_folder).expect("Testing shouldn't fail.");
        let item_file = item_folder.join("file.txt");
        fs::write(&item_file, "test").expect("Testing shouldn't fail.");
        path
    }

    #[test]
    fn new() -> Result<(), VersionError> {
        let patch_dir_path = patch_dir("new");
        let tracked_folder_path = tracked_folder_path("new");
        let tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path);
        assert!(tracked_folder.is_ok());
        Ok(())
    }

    #[test]
    fn save() -> Result<(), VersionError> {
        let patch_dir_path = patch_dir("save");
        let tracked_folder_path = tracked_folder_path("save");
        let mut tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path)?;
        tracked_folder.save()
    }

    #[test]
    fn load() -> Result<(), VersionError> {
        let patch_dir_path = patch_dir("load");
        let tracked_folder_path = tracked_folder_path("load");
        let mut tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path)?;
        tracked_folder.save()?;
        tracked_folder.save()?;
        tracked_folder.load(0)
    }

    #[test]
    fn delete() -> Result<(), VersionError> {
        let patch_dir_path = patch_dir("delete");
        let tracked_folder_path = tracked_folder_path("delete");
        let mut tracked_folder = TrackedFolder::new(&tracked_folder_path, &patch_dir_path)?;
        tracked_folder.save()?;
        tracked_folder.save()?;
        tracked_folder.delete(0)
    }
}
