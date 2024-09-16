use std::{fs, hash::{DefaultHasher, Hash, Hasher}, io::{self, Read}, path::{Path, PathBuf}};

use bzip2::{bufread::{BzDecoder, BzEncoder}, Compression};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct Patch {
    path: PathBuf,
}

impl Patch {

    pub fn new(source_path:  impl AsRef<Path>, target_path:  impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
        let source = std::fs::read(&source_path)?;
        let target = std::fs::read(&target_path)?;
        Self::from_buffers(&source, &target, patch_dir)
    }

    fn from_buffers(source: &[u8], target: &[u8], patch_dir: impl AsRef<Path>) -> io::Result<Self> {
        let mut patch_buffer = Vec::new();
        bsdiff::diff(source, target, &mut patch_buffer)?;
        let hash = Self::hash_buffer(&patch_buffer);
        let patch_path = Self::patch_path(patch_dir, hash);
        patch_buffer = Self::compress(&patch_buffer)?;
        fs::write(&patch_path, patch_buffer)?;
        Ok(Self {
            path: patch_path,
        })
    }

    fn hash_buffer(bytes: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }

    fn patch_path(patch_dir: impl AsRef<Path>, hash: u64) -> PathBuf {
        let mut path = patch_dir.as_ref().to_path_buf();
        path.push(format!("{}.ezpatch", hash));
        path
    }

    fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
        let mut encoder = BzEncoder::new(data, Compression::best());
        let mut compressed_data = vec![];
        encoder.read_to_end(&mut compressed_data)?;
        Ok(compressed_data)
    }

    fn decompress(data: &[u8]) -> io::Result<Vec<u8>> {
        let mut decoder = BzDecoder::new(data);
        let mut decompressed_data = vec![];
        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }

    pub fn apply_buffer(&self, source: &[u8]) -> io::Result<Vec<u8>> {
        let mut patch = fs::read(&self.path)?;
        patch = Self::decompress(&patch)?;
        let mut target_buffer = vec![];
        bsdiff::patch(source, &mut patch.as_slice(), &mut target_buffer)?;
        Ok(target_buffer)
    }

    pub fn apply_to_file(&self, source_path: impl AsRef<Path>, target_path: impl AsRef<Path>) -> io::Result<()> {
        let source = std::fs::read(&source_path)?;
        let target = self.apply_buffer(&source)?;
        std::fs::write(&target_path, target)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl From<Patch> for PathBuf {
    fn from(patch: Patch) -> Self {
        patch.path
    }
}

#[cfg(test)]
mod patch_tests {
    use super::*;

    #[test]
    fn test_patch() {
        let patch = Patch::new("test-data/patch/A.txt", "test-data/patch/B.txt", "test-data/patch");
        assert!(patch.is_ok());
    }

    #[test]
    fn test_apply() -> io::Result<()> {
        let source_path = "test-data/patch/A.txt";
        let target_path = "test-data/patch/B.txt";
        assert_eq!(fs::read_to_string(target_path)?, "ABCDEFG");
        let patch = Patch::new(source_path, target_path, "test-data/patch")?;
        fs::write(target_path, "")?;
        patch.apply_to_file(source_path, target_path)?;
        assert_eq!(fs::read_to_string(target_path)?, "ABCDEFG");
        Ok(())
    }
}
