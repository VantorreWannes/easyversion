use std::io::{self, Read, Write};

use bzip2::{
    bufread::{BzDecoder, BzEncoder},
    Compression,
};

use crate::hash;

#[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
pub struct Patch {
    data: Vec<u8>,
}

impl Patch {
    pub fn new(source: &[u8], target: &[u8]) -> io::Result<Self> {
        let mut data = Vec::new();
        bsdiff::diff(source, target, &mut data)?;
        let mut encoder = BzEncoder::new(data.as_slice(), Compression::best());
        let mut compressed_data = vec![];
        encoder.read_to_end(&mut compressed_data)?;
        Ok(Self {
            data: compressed_data,
        })
    }

    pub fn read_from<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Self { data })
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(&self.data)?;
        Ok(())
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn id(&self) -> u64 {
        hash(&self)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn apply(&self, source: &[u8]) -> io::Result<Vec<u8>> {
        let mut uncompressed_data = vec![];
        let mut decoder = BzDecoder::new(self.data.as_slice());
        decoder.read_to_end(&mut uncompressed_data)?;
        let mut data = vec![];
        bsdiff::patch(source, &mut uncompressed_data.as_slice(), &mut data)?;
        Ok(data)
    }

    pub fn from_data(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }
}

impl From<Patch> for Vec<u8> {
    fn from(patch: Patch) -> Self {
        patch.data
    }
}

impl From<&[u8]> for Patch {
    fn from(value: &[u8]) -> Self {
        Self::from_data(value)
    }
}

#[cfg(test)]
mod patch_tests {
    use io::Cursor;

    use super::*;

    #[test]
    fn new() {
        assert!(Patch::new(&[2], &[1, 2, 3]).is_ok());
    }

    #[test]
    fn apply() -> io::Result<()> {
        let source = [2];
        let target = [1, 2, 3];
        let patch = Patch::new(&source, &target)?;
        assert_eq!(patch.apply(&source)?, target);
        Ok(())
    }

    #[test]
    fn id() -> io::Result<()> {
        let patch = Patch::new(&[2], &[1, 2, 3])?;
        assert_eq!(patch.id(), 132369031730439770);
        Ok(())
    }

    #[test]
    fn write_to() -> io::Result<()> {
        let patch = Patch::from_data(&[2]);
        let mut file = Cursor::new(Vec::new());
        patch.write_to(&mut file)?;
        assert_eq!(file.into_inner(), [2]);
        Ok(())
    }

    #[test]
    fn read_from() {
        let data = [2];
        let patch = Patch::from_data(&data);
        assert_eq!(patch.data(), &data);
    }
}
