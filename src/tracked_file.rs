use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
};

use crate::tracked_bytes::TrackedBytes;

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    path: PathBuf,
    tracked_bytes: TrackedBytes,
}

impl TrackedFile {
    pub fn new(patch_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            tracked_bytes: TrackedBytes::new(patch_dir),
        }
    }

    pub fn load(&self, patch_index: usize) -> io::Result<()> {
        let target = self.tracked_bytes.load(patch_index)?;
        std::fs::write(&self.path, target)
    }

    pub fn save(&mut self) -> io::Result<()> {
        let target = std::fs::read(&self.path)?;
        self.tracked_bytes.save(&target)
    }
}

#[cfg(test)]
mod tracked_tests {
    use std::fs;

    use super::*;

    #[test]
    fn save() -> io::Result<()> {
        let tracked_file_path = "test-data/tracked-file/files/A.txt";
        let patches_dir = "test-data/tracked-file/patches";
        let mut tracked = TrackedFile::new(patches_dir, tracked_file_path);
        tracked.save()
    }

    #[test]
    fn load() -> io::Result<()> {
        let tracked_file_path = "test-data/tracked-file/files/B.txt";
        let patches_dir = "test-data/tracked-file/patches";
        let mut tracked = TrackedFile::new(patches_dir, tracked_file_path);

        fs::write(tracked_file_path, "1")?;
        tracked.save()?;
        fs::write(tracked_file_path, "2")?;
        tracked.save()?;

        tracked.load(0)?;
        assert_eq!(fs::read_to_string(tracked_file_path)?, "1");
        tracked.load(1)?;
        assert_eq!(fs::read_to_string(tracked_file_path)?, "2");
        Ok(())
    }

    #[test]
    fn new() {
        let tracked_file_path = "test-data/tracked-file/files/A.txt";
        let patches_dir = "test-data/tracked-file/patches";
        let _ = TrackedFile::new(patches_dir, tracked_file_path);
    }
}
