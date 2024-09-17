use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

use crate::patch::Patch;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct Version<T> {
    location: T,
}

impl<T> Version<T> {
    pub fn open(location: &mut T) -> io::Result<Patch>
    where
        T: Read,
    {
        let mut data = Vec::new();
        location.read_to_end(&mut data)?;
        Ok(Patch::from_data(&data))
    }

    pub fn write(mut location: T, patch: &Patch) -> io::Result<Self>
    where
        T: Write,
    {
        location.write_all(&patch.data())?;
        Ok(Self { location })
    }

    pub fn location(&self) -> &T {
        &self.location
    }
}

impl<T> From<T> for Version<T> {
    fn from(value: T) -> Self {
        Self { location: value }
    }
}

impl<T> AsRef<T> for Version<T> {
    fn as_ref(&self) -> &T {
        &self.location
    }
}

#[cfg(test)]
mod version_tests {

    use io::Cursor;

    use super::*;

    #[test]
    fn open_buffer() -> io::Result<()> {
        let patch = Version::open(&mut b"test".as_slice())?;
        assert_eq!(patch.data(), b"test");
        Ok(())
    }

    #[test]
    fn write_buffer() -> io::Result<()> {
        let patch = Patch::from_data(b"test");
        let mut location = Vec::new();
        let version = Version::write(&mut location, &patch)?;
        assert_eq!(version.location, b"test");
        Ok(())
    }

    #[test]
    fn open_file() -> io::Result<()> {
        let mut file = Cursor::new(b"test");
        let patch = Version::open(&mut file)?;
        assert_eq!(patch.data(), b"test");
        Ok(())
    }

    #[test]
    fn write_file() -> io::Result<()> {
        let patch = Patch::from_data(b"test");
        let mut file = Cursor::new(Vec::new());
        let version = Version::write(&mut file, &patch)?;
        assert_eq!(version.location.clone().into_inner(), b"test");
        assert_eq!(file.into_inner(), b"test");
        Ok(())
    }
}
