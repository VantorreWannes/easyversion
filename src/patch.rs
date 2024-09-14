use std::{hash::{DefaultHasher, Hash, Hasher}, io::{self, Read}, path::Path};
use bzip2::read::{BzEncoder, BzDecoder};
use bzip2::Compression;

#[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
struct Patch {
    data: Vec<u8>,
}

impl Patch {
    pub fn from_buffers(source: &[u8], target: &[u8]) -> io::Result<Self> {
        let mut patch_buffer = Vec::new();
        bsdiff::diff(source, target, &mut patch_buffer)?;
        Ok(Self { data: patch_buffer })
    }

    pub fn from_files(source_path: impl AsRef<Path>, target_path: impl AsRef<Path>) -> io::Result<Self> {
        let source = std::fs::read(source_path)?;
        let target = std::fs::read(target_path)?;
        Self::from_buffers(&source, &target)
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Self { data })
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::write(path, &self.data)
    }

    pub fn apply(&self, source: &[u8]) -> io::Result<Vec<u8>> {
        let mut target_buffer = Vec::new();
        bsdiff::patch(source, &mut self.data.as_slice(), &mut target_buffer)?;
        Ok(target_buffer)
    }

    pub fn apply_to_file(&self, source_path: impl AsRef<Path>, target_path: impl AsRef<Path>) -> io::Result<()> {
        let source = std::fs::read(source_path)?;
        let target = self.apply(&source)?;
        std::fs::write(target_path, target)
    }

    pub fn compress(&mut self) -> io::Result<()> {
        let mut encoder = BzEncoder::new(self.data.as_slice(), Compression::best());
        let mut compressed_data = Vec::new();
        encoder.read_to_end(&mut compressed_data)?;
        self.data = compressed_data;
        Ok(())
    }

    pub fn decompress(&mut self) -> io::Result<()> {
        let mut decoder = BzDecoder::new(self.data.as_slice());
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;
        self.data = decompressed_data;
        Ok(())
    }

    pub fn id(&self) -> u64 {
        let mut hash_state = DefaultHasher::new();
        self.hash(&mut hash_state);
        hash_state.finish()
    }
}

impl From<Vec<u8>> for Patch {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl From<Patch> for Vec<u8> {
    fn from(patch: Patch) -> Self {
        patch.data
    }
}

impl TryFrom<&Path> for Patch {
    type Error = io::Error;
    
    fn try_from(path: &Path) -> Result<Self, std::io::Error> {
        Self::open(path)
    }    
}

#[cfg(test)]
mod patch_tests {
    use super::*;

    #[test]
    fn from_buffers() {
        let source = vec![1, 2, 3, 4, 5];
        let target = vec![1, 2, 4, 6];
        let patch = Patch::from_buffers(&source, &target);
        assert!(patch.is_ok());
    }

    #[test]
    fn from_files() {
        let source = "test-data/patch/A.txt";
        let target = "test-data/patch/B.txt";
        let patch = Patch::from_files(source, target);
        assert!(patch.is_ok());
    }

    #[test]
    fn save() -> io::Result<()> {
        let source = vec![1, 2, 3, 4, 5];
        let target = vec![1, 2, 4, 6];
        let patch = Patch::from_buffers(&source, &target)?;
        let path = "test-data/patch/patch.ezpatch";
        patch.save(path)
    }

    #[test]
    fn open() {
        let path = "test-data/patch/patch.ezpatch";
        let patch = Patch::open(path);
        assert!(patch.is_ok());
    }

    #[test]
    fn apply() -> io::Result<()> {
        let source = vec![1, 2, 3, 4, 5];
        let target = vec![1, 2, 4, 6];
        let patch = Patch::from_buffers(&source, &target).unwrap();
        let result = patch.apply(&source)?;
        assert_eq!(target, result);
        Ok(())
    }

    #[test]
    fn apply_to_file() -> io::Result<()> {
        let source = "test-data/patch/A.txt";
        let target = "test-data/patch/B.txt";
        let new_target = "test-data/patch/C.txt";
        let patch = Patch::from_files(source, target)?;
        patch.apply_to_file(source, new_target)
    }

    #[test]
    fn id() -> io::Result<()> {
        let source = vec![1, 2, 3, 4, 5];
        let target = vec![1, 2, 4, 6];
        let patch = Patch::from_buffers(&source, &target)?;
        assert_eq!(patch.id(), 3130703799529806172);
        Ok(())
    }

    #[test]
    fn compress() -> io::Result<()> {
        let source = vec![1, 2, 3, 4, 5];
        let target = vec![1, 2, 4, 6];
        let uncompressed_patch = Patch::from_buffers(&source, &target)?;
        let mut compressed_patch = uncompressed_patch.clone();
        compressed_patch.compress()?;
        assert_ne!(uncompressed_patch, compressed_patch);
        Ok(())
    }

    #[test]
    fn decompress() -> io::Result<()> {
        let source = vec![1, 2, 3, 4, 5];
        let target = vec![1, 2, 4, 6];
        let uncompressed_patch = Patch::from_buffers(&source, &target)?;

        let mut compressed_patch = uncompressed_patch.clone();
        compressed_patch.compress()?;

        let mut decompressed_patch = compressed_patch.clone();
        decompressed_patch.decompress()?;

        assert_eq!(uncompressed_patch, decompressed_patch);
        Ok(())
    }
}
