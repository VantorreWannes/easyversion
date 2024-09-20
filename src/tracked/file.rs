use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{patch::Patch, timeline::Timeline};

use super::Version;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    path: PathBuf,
    timeline: Timeline,
}

impl TrackedFile {
    pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
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

    fn apply(&self, index: usize) -> io::Result<Vec<u8>> {
        let mut source = vec![];
        for index in 0..=index {
            if let Some(patch) = self.timeline.get(index).transpose()? {
                source = patch.apply(&source)?;
            }
        }
        Ok(source)
    }

    fn apply_all(&self) -> io::Result<Vec<u8>> {
        self.apply(self.timeline.len().saturating_sub(1))
    }
}

impl Version for TrackedFile {
    fn save(&mut self) -> std::io::Result<()> {
        let source = self.apply_all()?;
        let target = fs::read(&self.path)?;
        let patch = Patch::new(&source, &target)?;
        self.timeline.push(&patch)
    }

    fn load(&mut self, index: usize) -> std::io::Result<()> {
        let target = self.apply(index)?;
        fs::write(&self.path, &target)
    }

    fn delete(&mut self, index: usize) -> std::io::Result<()> {
        self.load(index)?;
        for _ in index..self.timeline.len() {
            self.timeline.pop().transpose()?;
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.timeline.len()
    }
}

#[cfg(test)]
mod tracked_file_tests {
    use std::io;

    use crate::tracked::{
        version_test_tools::{patch_dir_path, tracked_file_path},
        Version,
    };

    use super::TrackedFile;

    #[test]
    fn new() -> io::Result<()> {
        let dirs = &["tracked_file", "new"];
        let patch_dir = patch_dir_path(dirs)?;
        let tracked_file_path = tracked_file_path(dirs)?;
        let tracked_file = TrackedFile::new(tracked_file_path, patch_dir);
        assert!(tracked_file.is_ok());
        Ok(())
    }

    #[test]
    fn save() -> io::Result<()> {
        let dirs = &["tracked_file", "save"];
        let patch_dir = patch_dir_path(dirs)?;
        let tracked_file_path = tracked_file_path(dirs)?;
        let mut tracked_file = TrackedFile::new(tracked_file_path, patch_dir)?;
        tracked_file.save()
    }

    #[test]
    fn load() -> io::Result<()> {
        let dirs = &["tracked_file", "load"];
        let patch_dir = patch_dir_path(dirs)?;
        let tracked_file_path = tracked_file_path(dirs)?;
        let mut tracked_file = TrackedFile::new(tracked_file_path, patch_dir)?;
        tracked_file.save()?;
        tracked_file.load(0)
    }

    #[test]
    fn delete() -> io::Result<()> {
        let dirs = &["tracked_file", "delete"];
        let patch_dir = patch_dir_path(dirs)?;
        let tracked_file_path = tracked_file_path(dirs)?;
        let mut tracked_file = TrackedFile::new(tracked_file_path, patch_dir)?;
        tracked_file.save()?;
        tracked_file.delete(0)
    }
}
