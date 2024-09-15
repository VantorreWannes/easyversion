use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
};

use crate::patch::Patch;

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct TrackedBytes {
    patch_dir: PathBuf,
    patch_paths: Vec<PathBuf>,
}

impl TrackedBytes {
    pub fn new(patch_dir: impl AsRef<Path>) -> Self {
        Self {
            patch_paths: vec![],
            patch_dir: patch_dir.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self, patch_index: usize) -> io::Result<Vec<u8>> {
        Self::load_patches(&self.patch_paths[..=patch_index])
    }

    pub fn save(&mut self, target: &[u8]) -> io::Result<()> {
        let source = Self::load_patches(&self.patch_paths)?;
        self.save_patch(&source, target)
    }

    fn load_patches(patch_paths: &[impl AsRef<Path>]) -> io::Result<Vec<u8>> {
        let mut source = vec![];
        for patch in patch_paths {
            source = Self::apply_patch(&source, patch)?;
        }
        Ok(source)
    }

    fn apply_patch(source: &[u8], patch_path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let mut patch = Patch::open(patch_path)?;
        patch.decompress()?;
        let target = patch.apply(source)?;
        Ok(target)
    }

    fn save_patch(&mut self, source: &[u8], target: &[u8]) -> io::Result<()> {
        let mut patch = Patch::from_buffers(&source, target)?;
        patch.compress()?;
        patch.save(&self.patch_dir)?;
        self.patch_paths.push(patch.path(&self.patch_dir));
        Ok(())
    }
}

#[cfg(test)]
mod tracked_bytes_tests {
    use super::*;

    #[test]
    fn save() {
        let bytes = vec![0; 10];
        let mut tracked = TrackedBytes::new("test-data/tracked-bytes/patches");
        assert!(tracked.save(&bytes).is_ok());
    }

    #[test]
    fn load() -> io::Result<()> {
        let mut bytes = vec![0; 5];
        let mut tracked = TrackedBytes::new("test-data/tracked-bytes/patches");
        tracked.save(&bytes)?;
        bytes[0] = 1;
        tracked.save(&bytes)?;
        assert_eq!(&bytes, &vec![1, 0, 0, 0, 0]);
        let loaded = tracked.load(0)?;
        assert_eq!(&loaded, &vec![0, 0, 0, 0, 0]);
        Ok(())
    }

    #[test]
    fn new() {
        let patch_dir = Path::new("test-data/tracked-bytes/patches");
        let _ = TrackedBytes::new(patch_dir);
    }
}
