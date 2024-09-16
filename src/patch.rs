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
        let mut compressed_data = Vec::new();
        encoder.read_to_end(&mut compressed_data)?;
        Ok(compressed_data)
    }

    fn decompress(data: &[u8]) -> io::Result<Vec<u8>> {
        let mut decoder = BzDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
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
}
