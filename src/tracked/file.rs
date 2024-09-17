use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::patch::Patch;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    file_path: PathBuf,
    patch_dir: PathBuf,
    patches: Vec<Patch>,
}

impl TrackedFile {
    pub fn new(file_path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
            patch_dir: patch_dir.as_ref().to_path_buf(),
            patches: vec![],
        }
    }

    pub fn save(&mut self) -> io::Result<()> {
        let target = fs::read(&self.file_path)?;
        self.save_buffer(&target)?;
        Ok(())
    }

    fn save_buffer(&mut self, target: &[u8]) -> io::Result<()> {
        let source = self.load_buffer(self.patches.len())?;
        let patch = Patch::from_buffers(&source, target, &self.patch_dir)?;
        self.patches.push(patch);
        Ok(())
    }

    pub fn load(&mut self, index: usize) -> io::Result<()> {
        let target = self.load_buffer(index)?;
        fs::write(&self.file_path, &target)
    }

    fn load_buffer(&self, index: usize) -> io::Result<Vec<u8>> {
        let mut source = vec![];
        if self.patches.is_empty() || index == usize::MAX {
            return Ok(source);
        }
        for patch in self.patches.iter().take(index + 1) {
            source = patch.apply_buffer(&source)?;
        }
        Ok(source)
    }

    pub fn delete(&mut self, index: usize) -> io::Result<()> {
        self.patches.truncate(index);
        Ok(())
    }
}

#[cfg(test)]
mod tracked_file_tests {
    use super::*;

    #[test]
    fn test_save() -> io::Result<()> {
        let mut tracked_file =
            TrackedFile::new("test-data/tracked/file/A.txt", "test-data/tracked/file");
        tracked_file.save()
    }

    #[test]
    fn test_load() -> io::Result<()> {
        let file_path = "test-data/tracked/file/B.txt";
        let mut tracked_file = TrackedFile::new(file_path, "test-data/tracked/file");
        fs::write(file_path, b"1")?;
        tracked_file.save()?;
        fs::write(file_path, b"2")?;
        tracked_file.load(0)?;
        assert_eq!(fs::read(file_path)?, b"1");
        Ok(())
    }

    #[test]
    fn test_delete() -> io::Result<()> {
        let file_path = "test-data/tracked/file/C.txt";
        let mut tracked_file = TrackedFile::new(file_path, "test-data/tracked/file");
        fs::write(file_path, b"1")?;
        tracked_file.save()?;
        tracked_file.save()?;
        assert_eq!(tracked_file.patches.len(), 2);
        tracked_file.delete(1)?;
        assert_eq!(tracked_file.patches.len(), 1);
        Ok(())
    }
}
