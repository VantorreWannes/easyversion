use std::{error::Error, fmt::Display};

use file::TrackedFile;
use folder::TrackedFolder;
use serde::{Deserialize, Serialize};

use crate::patches::patch_timeline::PatchTimelineError;

pub mod file;
pub mod folder;

#[derive(Debug)]
pub enum VersionError {
    PatchTimelineError(PatchTimelineError),
}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::PatchTimelineError(patch_timeline_error) => {
                write!(f, "{}", patch_timeline_error)
            }
        }
    }
}

impl Error for VersionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VersionError::PatchTimelineError(err) => Some(err),
        }
    }
}

impl From<PatchTimelineError> for VersionError {
    fn from(err: PatchTimelineError) -> Self {
        Self::PatchTimelineError(err)
    }
}

pub trait Version {
    /// Commits the current state as a new version.
    fn commit(&mut self) -> Result<(), VersionError>;

    /// Loads the state from the version at the given index.
    fn load_version(&self, index: usize) -> Result<(), VersionError>;

    /// Deletes the version at the given index.
    fn delete_version(&mut self, index: usize) -> Result<(), VersionError>;

    /// Returns the total number of versions.
    fn version_count(&self) -> usize;

    /// Checks if there are no versions saved.
    fn is_empty(&self) -> bool {
        self.version_count() == 0
    }

    /// Retrieves the index of the latest version.
    fn latest_version_index(&self) -> Option<usize> {
        match self.version_count() {
            0 => None,
            len => Some(len - 1),
        }
    }

    /// Loads the latest version.
    fn load_latest(&mut self) -> Result<(), VersionError> {
        match self.latest_version_index() {
            Some(index) => self.load_version(index),
            None => Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable,
            )),
        }
    }

    /// Deletes the latest version.
    fn delete_latest(&mut self) -> Result<(), VersionError> {
        match self.latest_version_index() {
            Some(index) => self.delete_version(index),
            None => Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable,
            )),
        }
    }

    /// Replaces the latest version with the current state.
    fn replace_latest(&mut self) -> Result<(), VersionError> {
        self.delete_latest()?;
        self.commit()
    }

    /// Reverts to the latest saved version.
    fn revert(&mut self) -> Result<(), VersionError> {
        self.load_latest()
    }

    /// Deletes all saved versions.
    fn clear_versions(&mut self) -> Result<(), VersionError> {
        for _ in 0..self.version_count() {
            self.delete_latest()?;
        }
        Ok(())
    }

    /// Creates a new instance starting from the currently loaded version.
    fn fork(&self) -> Result<Self, VersionError>
    where
        Self: Sized + Clone,
    {
        let mut new_instance = self.clone();
        new_instance.clear_versions()?;
        new_instance.commit()?;
        Ok(new_instance)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub enum TrackedItem {
    File(TrackedFile),
    Folder(TrackedFolder),
}

impl From<TrackedFile> for TrackedItem {
    fn from(file: TrackedFile) -> Self {
        Self::File(file)
    }
}

impl From<TrackedFolder> for TrackedItem {
    fn from(folder: TrackedFolder) -> Self {
        Self::Folder(folder)
    }
}

impl TrackedItem {
    pub fn file(&self) -> Option<&TrackedFile> {
        match self {
            Self::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn folder(&self) -> Option<&TrackedFolder> {
        match self {
            Self::Folder(folder) => Some(folder),
            _ => None,
        }
    }
}

impl Version for TrackedItem {
    fn commit(&mut self) -> Result<(), VersionError> {
        match self {
            Self::File(file) => file.commit(),
            Self::Folder(folder) => folder.commit(),
        }
    }

    fn load_version(&self, index: usize) -> Result<(), VersionError> {
        match self {
            Self::File(file) => file.load_version(index),
            Self::Folder(folder) => folder.load_version(index),
        }
    }

    fn delete_version(&mut self, index: usize) -> Result<(), VersionError> {
        match self {
            Self::File(file) => file.delete_version(index),
            Self::Folder(folder) => folder.delete_version(index),
        }
    }

    fn version_count(&self) -> usize {
        match self {
            Self::File(file) => file.version_count(),
            Self::Folder(folder) => folder.version_count(),
        }
    }
}
