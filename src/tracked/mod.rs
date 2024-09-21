use std::{error::Error, fmt::Display, io, path::Path};

// use file::TrackedFile;
// use folder::TrackedFolder;
use serde::{Deserialize, Serialize};

use crate::{patch::PatchError, timeline::TimelineError};
pub mod file;
// pub mod folder;

// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// pub enum TrackedItem {
//     File(file::TrackedFile),
//     Folder(folder::TrackedFolder),
// }

// impl TrackedItem {
//     pub fn new(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
//         if path.as_ref().is_file() {
//             Ok(Self::File(file::TrackedFile::new(path, patch_dir)?))
//         } else if path.as_ref().is_dir() {
//             Ok(Self::Folder(folder::TrackedFolder::new(path, patch_dir)?))
//         } else {
//             Err(io::Error::new(io::ErrorKind::Other, "Not a file or folder"))
//         }
//     }

//     pub fn file(&self) -> Option<&TrackedFile> {
//         match self {
//             TrackedItem::File(file) => Some(file),
//             _ => None,
//         }
//     }

//     pub fn folder(&self) -> Option<&TrackedFolder> {
//         match self {
//             TrackedItem::Folder(folder) => Some(folder),
//             _ => None,
//         }
//     }
// }

// impl Version for TrackedItem {
//     fn save(&mut self) -> io::Result<()> {
//         match self {
//             TrackedItem::File(file) => file.save(),
//             TrackedItem::Folder(folder) => folder.save(),
//         }
//     }

//     fn load(&mut self, index: usize) -> io::Result<()> {
//         match self {
//             TrackedItem::File(file) => file.load(index),
//             TrackedItem::Folder(folder) => folder.load(index),
//         }
//     }

//     fn delete(&mut self, index: usize) -> io::Result<()> {
//         match self {
//             TrackedItem::File(file) => file.delete(index),
//             TrackedItem::Folder(folder) => folder.delete(index),
//         }
//     }

//     fn len(&self) -> usize {
//         match self {
//             TrackedItem::File(file) => file.len(),
//             TrackedItem::Folder(folder) => folder.len(),
//         }
//     }
// }

#[derive(Debug)]
pub enum VersionError {
    TimelineError(TimelineError),
    IndexOutOfRange(usize),
}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::TimelineError(err) => err.fmt(f),
            VersionError::IndexOutOfRange(idx) => write!(f, "Index out of range: {}", idx),
        }
    }
}

impl Error for VersionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VersionError::TimelineError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<TimelineError> for VersionError {
    fn from(err: TimelineError) -> Self {
        VersionError::TimelineError(err)
    }
}

impl From<PatchError> for VersionError {
    fn from(err: PatchError) -> Self {
        VersionError::TimelineError(err.into())
    }
}

impl From<io::Error> for VersionError {
    fn from(err: io::Error) -> Self {
        VersionError::TimelineError(err.into())
    }
}

pub trait Version {
    fn save(&mut self) -> Result<(), VersionError>;

    fn load(&mut self, index: usize) -> Result<(), VersionError>;

    fn delete(&mut self, index: usize) -> Result<(), VersionError>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn last_index(&self) -> usize {
        self.len().saturating_sub(1)
    }

    fn load_last(&mut self) -> Result<(), VersionError> {
        self.load(self.last_index())
    }

    fn delete_last(&mut self) -> Result<(), VersionError> {
        self.delete(self.last_index())
    }

    fn restore(&mut self) -> Result<(), VersionError> {
        self.load_last()
    }

    fn clear(&mut self) -> Result<(), VersionError> {
        self.delete(0)
    }

    fn split(&mut self, index: usize) -> Result<Self, VersionError>
    where
        Self: Sized + Clone,
    {
        self.load(index)?;
        let mut other = self.clone();
        other.clear()?;
        other.save()?;
        Ok(other)
    }
}
