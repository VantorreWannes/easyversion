use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{hash, patch::Patch, timeline::Timeline};

use super::{Version, VersionError};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    path: PathBuf,
    timeline: Timeline,
}

impl TrackedFile {
    pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> Result<Self, VersionError> {
        let patch_dir = patch_dir.as_ref().join(hash(path.as_ref()).to_string());
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            timeline: Timeline::new(patch_dir)?,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    fn apply(&self, index: usize) -> Result<Vec<u8>, VersionError> {
        if index > self.timeline.len() {
            return Err(VersionError::IndexOutOfRange(index));
        }
        let mut source = vec![];
        for index in 0..=index {
            if let Some(patch) = self.timeline.get(index).transpose()? {
                source = patch.apply(&source)?;
            }
        }
        Ok(source)
    }

    fn apply_all(&self) -> Result<Vec<u8>, VersionError> {
        self.apply(self.timeline.len().saturating_sub(1))
    }
}

impl Version for TrackedFile {
    fn save(&mut self) -> Result<(), VersionError> {
        let source = self.apply_all()?;
        let target = fs::read(&self.path)?;
        let patch = Patch::new(&source, &target)?;
        Ok(self.timeline.push(&patch)?)
    }

    fn load(&mut self, index: usize) -> Result<(), VersionError> {
        let target = self.apply(index)?;
        Ok(fs::write(&self.path, &target)?)
    }

    fn delete(&mut self, index: usize) -> Result<(), VersionError> {
        self.load(index)?;
        for _ in index..self.timeline.len() {
            self.timeline.pop()?;
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.timeline.len()
    }
}

#[cfg(test)]
mod tracked_file_tests {
    use crate::test_tools::dir_path;

    use super::*;

    fn patch_dir(name: &str) -> PathBuf {
        dir_path(&["tracked_file", "patches", name])
    }

    fn tracked_file_path(name: &str) -> PathBuf {
        let path = dir_path(&["tracked_file", "items", name]).join("file.txt");
        fs::write(&path, "test").expect("Testing shouldn't fail.");
        path
    }

    #[test]
    fn new() -> Result<(), VersionError> {
        let patch_dir = patch_dir("new");
        let tracked_file_path = tracked_file_path("new");
        let tracked_file = TrackedFile::new(tracked_file_path, patch_dir);
        assert!(tracked_file.is_ok());
        Ok(())
    }

    #[test]
    fn save() -> Result<(), VersionError> {
        let patch_dir = patch_dir("save");
        let tracked_file_path = tracked_file_path("save");
        let mut tracked_file = TrackedFile::new(tracked_file_path, patch_dir)?;
        tracked_file.save()
    }

    #[test]
    fn load() -> Result<(), VersionError> {
        let patch_dir = patch_dir("load");
        let tracked_file_path = tracked_file_path("load");
        let mut tracked_file = TrackedFile::new(tracked_file_path, patch_dir)?;
        tracked_file.save()?;
        tracked_file.save()?;
        tracked_file.load(0)
    }

    #[test]
    fn delete() -> Result<(), VersionError> {
        let patch_dir = patch_dir("delete");
        let tracked_file_path = tracked_file_path("delete");
        let mut tracked_file = TrackedFile::new(tracked_file_path, patch_dir)?;
        tracked_file.save()?;
        tracked_file.save()?;
        tracked_file.delete(0)
    }
}
