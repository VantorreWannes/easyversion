use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{patch::Patch, timeline::Timeline};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    path: PathBuf,
    timeline: Timeline,
}

impl TrackedFile {
    pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            timeline: Timeline::new(patch_dir),
        }
    }

    pub fn file_path(&self) -> &Path {
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
        let index = self.timeline.len().saturating_sub(1);
        self.apply(index)
    }

    pub fn load(&mut self, index: usize) -> io::Result<()> {
        let target = self.apply(index)?;
        fs::write(&self.path, &target)?;
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        let source = self.apply_all()?;
        let target = fs::read(&self.path)?;
        let patch = Patch::new(&source, &target)?;
        self.timeline.push(&patch)?;
        Ok(())
    }
}

#[cfg(test)]
mod tracked_file_tests {

    use std::path::PathBuf;

    use super::*;

    fn patch_dir(name: &str) -> io::Result<PathBuf> {
        let patch_dir = dirs::config_dir()
            .unwrap()
            .join("easyversion")
            .join("tests")
            .join("tracked_file")
            .join(name);
        std::fs::create_dir_all(&patch_dir)?;
        Ok(patch_dir)
    }

    #[test]
    fn new() -> io::Result<()> {
        let patch_dir = patch_dir("new")?;
        let file_path = patch_dir.join("file.txt");
        fs::write(&file_path, b"")?;
        let _ = TrackedFile::new(file_path, patch_dir);
        Ok(())
    }

    #[test]
    fn save() -> io::Result<()> {
        let patch_dir = patch_dir("save")?;
        let file_path = patch_dir.join("file.txt");
        fs::write(&file_path, b"123")?;
        let mut tracked_file = TrackedFile::new(file_path, patch_dir);
        tracked_file.save()?;
        Ok(())
    }

    #[test]
    fn load() -> io::Result<()> {
        let patch_dir = patch_dir("load")?;
        let file_path = patch_dir.join("file.txt");
        fs::write(&file_path, b"123")?;
        let mut tracked_file = TrackedFile::new(file_path, patch_dir);
        tracked_file.save()?;
        tracked_file.load(0)?;
        Ok(())
    }
}
