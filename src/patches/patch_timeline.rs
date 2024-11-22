use std::{
    error::Error,
    fmt::Display,
    fs,
    hash::Hash,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::hash;

use super::patch::{Patch, PatchError};

#[derive(Debug)]
pub enum PatchTimelineError {
    IoError(io::Error),
    PatchError(PatchError),
    IndexOutOfRange(usize),
    NoVersionsAvailable,
}

impl Display for PatchTimelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTimelineError::IoError(err) => err.fmt(f),
            PatchTimelineError::PatchError(err) => err.fmt(f),
            PatchTimelineError::IndexOutOfRange(idx) => {
                write!(f, "Patch index is out of range: {}", idx)
            }
            PatchTimelineError::NoVersionsAvailable => write!(f, "No versions available"),
        }
    }
}

impl Error for PatchTimelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PatchTimelineError::IoError(err) => Some(err),
            PatchTimelineError::PatchError(err) => Some(err),
            PatchTimelineError::IndexOutOfRange(_) => None,
            PatchTimelineError::NoVersionsAvailable => None,
        }
    }
}

impl From<io::Error> for PatchTimelineError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<PatchError> for PatchTimelineError {
    fn from(err: PatchError) -> Self {
        Self::PatchError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct PatchTimeline {
    dir: PathBuf,
    hashes: Vec<u64>,
}

impl PatchTimeline {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self, PatchTimelineError> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            dir: dir.as_ref().to_path_buf(),
            hashes: Vec::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    fn patch_path(&self, hash: u64) -> PathBuf {
        self.dir.join(Patch::filename(hash))
    }

    pub fn push(&mut self, patch: &Patch) -> Result<(), PatchTimelineError> {
        let hash = hash(patch);
        let path = self.patch_path(hash);
        if !path.exists() {
            let mut file = std::fs::File::create(&path)?;
            patch.write_to(&mut file)?;
        }
        self.hashes.push(hash);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), PatchTimelineError> {
        match self.hashes.pop() {
            Some(hash) => {
                if !self.hashes.contains(&hash) {
                    let path = self.patch_path(hash);
                    fs::remove_file(&path)?;
                }
            }
            None => return Err(PatchTimelineError::NoVersionsAvailable),
        }
        Ok(())
    }

    pub fn get(&self, idx: usize) -> Result<Patch, PatchTimelineError> {
        let hash = self
            .hashes
            .get(idx)
            .ok_or(PatchTimelineError::IndexOutOfRange(idx))?;
        let path = self.patch_path(*hash);
        let mut file = std::fs::File::open(&path)?;
        Ok(Patch::read_from(&mut file)?)
    }
}

#[cfg(test)]
mod patch_tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn new() -> Result<(), PatchTimelineError> {
        let patch_dir = TempDir::new("easyversion")?;
        let timeline = PatchTimeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        assert_eq!(timeline.len(), 0);
        Ok(())
    }

    #[test]
    fn push() -> Result<(), PatchTimelineError> {
        let patch_dir = TempDir::new("easyversion")?;
        let patch = Patch::from_data(&[]);
        let mut timeline = PatchTimeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        Ok(())
    }

    #[test]
    fn pop() -> Result<(), PatchTimelineError> {
        let patch_dir = TempDir::new("easyversion")?;
        let patch = Patch::from_data(&[]);
        let mut timeline = PatchTimeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        timeline.pop()?;
        assert!(timeline.is_empty());
        Ok(())
    }

    #[test]
    fn get() -> Result<(), PatchTimelineError> {
        let patch_dir = TempDir::new("easyversion")?;
        let patch = Patch::from_data(&[2]);
        let mut timeline = PatchTimeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        let gotten_patch = timeline.get(0)?;
        assert_eq!(gotten_patch, patch);
        Ok(())
    }
}
