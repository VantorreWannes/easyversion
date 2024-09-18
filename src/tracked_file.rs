use std::{
    io::{self, Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{patch::Patch, timeline::Timeline};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFile<T> {
    file: T,
    timeline: Timeline,
}

impl<T: Write + Read> TrackedFile<T> {
    pub fn new(file: T, patch_dir: impl AsRef<Path>) -> Self {
        Self {
            file,
            timeline: Timeline::new(patch_dir),
        }
    }

    pub fn file(&self) -> &T {
        &self.file
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
        self.file.write_all(&target)?;
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        let source = self.apply_all()?;
        let mut target = vec![];
        self.file.read_to_end(&mut target)?;
        let patch = Patch::new(&source, &target)?;
        self.timeline.push(&patch)?;
        Ok(())
    }
}

#[cfg(test)]
mod tracked_file_tests {

    use std::path::PathBuf;

    use io::Cursor;

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
        let file = Cursor::new(vec![]);
        let patch_dir = patch_dir("new")?;
        let _ = TrackedFile::new(file, patch_dir);
        Ok(())
    }

    #[test]
    fn save() -> io::Result<()> {
        let mut data = vec![1, 2, 3];
        let file = Cursor::new(&mut data);
        let patch_dir = patch_dir("save")?;
        let mut tracked_file = TrackedFile::new(file, patch_dir);
        tracked_file.save()?;
        Ok(())
    }

    #[test]
    fn load() -> io::Result<()> {
        let mut data = vec![1, 2, 3];
        let file = Cursor::new(&mut data);
        let patch_dir = patch_dir("load")?;
        let mut tracked_file = TrackedFile::new(file, patch_dir);
        tracked_file.save()?;
        tracked_file.load(0)?;
        Ok(())
    }
}
