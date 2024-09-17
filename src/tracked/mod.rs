use std::io;

use file::TrackedFile;
use folder::TrackedFolder;
use serde::{Deserialize, Serialize};

pub mod file;
pub mod folder;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TrackedItem {
    File(TrackedFile),
    Folder(TrackedFolder),
}

impl TrackedItem {
    pub fn save(&mut self) -> io::Result<()> {
        match self {
            Self::File(file) => file.save(),
            Self::Folder(folder) => folder.save(),
        }
    }

    pub fn load(&mut self, index: usize) -> io::Result<()> {
        match self {
            Self::File(file) => file.load(index),
            Self::Folder(folder) => folder.load(index),
        }
    }

    pub fn delete(&mut self, index: usize) -> io::Result<()> {
        match self {
            Self::File(file) => file.delete(index),
            Self::Folder(folder) => folder.delete(index),
        }
    }
}

impl From<TrackedFile> for TrackedItem {
    fn from(item: TrackedFile) -> Self {
        Self::File(item)
    }
}

impl From<TrackedFolder> for TrackedItem {
    fn from(item: TrackedFolder) -> Self {
        Self::Folder(item)
    }
}
