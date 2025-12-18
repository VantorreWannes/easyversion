use std::hash::Hasher;
use std::io::{self, Read};

pub mod id;
pub mod storage;

pub struct HashingReader<R, H> {
    inner: R,
    hasher: H,
}

impl<R: Read, H: Hasher> HashingReader<R, H> {
    pub fn new(inner: R, hasher: H) -> Self {
        Self { inner, hasher }
    }

    pub fn finalize(self) -> u64 {
        self.hasher.finish()
    }
}

impl<R: Read, H: Hasher> Read for HashingReader<R, H> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.write(&buf[..n]);
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SINGLE_BYTE_DATA: [u8; 1] = [42];
    const SINGLE_BYTE_DATA_HASH: u64 = 42;
    const LARGE_DATA_SIZE: usize = 100;

    fn sum_hasher(initial: u64) -> impl Hasher {
        struct SumHasher(u64);
        impl Hasher for SumHasher {
            fn finish(&self) -> u64 {
                self.0
            }
            fn write(&mut self, bytes: &[u8]) {
                for &byte in bytes {
                    self.0 += byte as u64;
                }
            }
        }
        impl Default for SumHasher {
            fn default() -> Self {
                Self(0)
            }
        }
        SumHasher(initial)
    }

    #[test]
    fn test_new_creates_instance() {
        let reader = Cursor::new(SINGLE_BYTE_DATA);
        let hasher = sum_hasher(0);

        let _hashing_reader = HashingReader::new(reader, hasher);
    }

    #[test]
    fn test_read_preserves_data() -> io::Result<()> {
        let mut reader = HashingReader::new(Cursor::new(SINGLE_BYTE_DATA), sum_hasher(0));
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer)?;

        assert_eq!(buffer, SINGLE_BYTE_DATA);
        Ok(())
    }

    #[test]
    fn test_finalize_computes_hash() -> io::Result<()> {
        let mut reader = HashingReader::new(Cursor::new(SINGLE_BYTE_DATA), sum_hasher(0));
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let hash = reader.finalize();

        assert_eq!(hash, SINGLE_BYTE_DATA_HASH);
        Ok(())
    }

    #[test]
    fn test_handles_large_data() -> io::Result<()> {
        let large_data = vec![1u8; LARGE_DATA_SIZE];
        let expected_hash = 1u64 * LARGE_DATA_SIZE as u64;
        let mut reader = HashingReader::new(Cursor::new(&large_data), sum_hasher(0));
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        assert_eq!(buffer, large_data);
        let hash = reader.finalize();
        assert_eq!(hash, expected_hash);
        Ok(())
    }
}
