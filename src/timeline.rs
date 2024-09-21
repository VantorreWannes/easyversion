use std::{
    error::Error,
    fmt::Display,
    fs,
    io::{self},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::patch::{Patch, PatchError};

#[derive(Debug)]
pub enum TimelineError {
    IoError(io::Error),
    PatchError(PatchError),
    IndexOutOfRange(usize),
}

impl Display for TimelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimelineError::IoError(err) => err.fmt(f),
            TimelineError::PatchError(err) => err.fmt(f),
            TimelineError::IndexOutOfRange(idx) => write!(f, "Index out of range: {}", idx),
        }
    }
}

impl Error for TimelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TimelineError::IoError(err) => Some(err),
            TimelineError::PatchError(err) => Some(err),
            TimelineError::IndexOutOfRange(_) => None,
        }
    }
}

impl From<io::Error> for TimelineError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<PatchError> for TimelineError {
    fn from(err: PatchError) -> Self {
        Self::PatchError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Timeline {
    patch_dir: PathBuf,
    patch_paths: Vec<PathBuf>,
}

impl Timeline {
    pub fn new(patch_dir: impl AsRef<Path>) -> Result<Self, TimelineError> {
        std::fs::create_dir_all(&patch_dir)?;
        Ok(Self {
            patch_dir: patch_dir.as_ref().to_path_buf(),
            patch_paths: vec![],
        })
    }

    pub fn push(&mut self, patch: &Patch) -> Result<(), TimelineError> {
        let patch_path = self.patch_dir.join(format!("{}.bz2", patch.id()));
        let mut patch_file = std::fs::File::create(&patch_path)?;
        patch.write_to(&mut patch_file)?;
        self.patch_paths.push(patch_path);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), TimelineError> {
        if let Some(patch_path) = self.patch_paths.pop() {
            if !self.patch_paths.contains(&patch_path) {
                fs::remove_file(&patch_path)?;
            }
        }
        Ok(())
    }

    pub fn trunicate(&mut self, index: usize) -> Result<(), TimelineError> {
        if index > self.patch_paths.len() {
            return Err(TimelineError::IndexOutOfRange(index));
        }
        for _ in index..self.patch_paths.len() {
            self.pop()?;
        }
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<Result<Patch, TimelineError>> {
        self.patch_paths.get(index).map(|path| {
            let patch_file = std::fs::File::open(path)?;
            Patch::read_from(&patch_file).map_err(|err| err.into())
        })
    }

    pub fn len(&self) -> usize {
        self.patch_paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patch_paths.is_empty()
    }

    pub fn patch_paths(&self) -> &[PathBuf] {
        &self.patch_paths
    }

    pub fn patch_dir(&self) -> &Path {
        &self.patch_dir
    }
}

#[cfg(test)]
mod timeline_tests {

    use crate::test_tools::dir_path;

    use super::*;

    pub fn patch_dir(name: &str) -> PathBuf {
        dir_path(&["timeline", name])
    }

    #[test]
    fn new() -> Result<(), TimelineError> {
        let patch_dir = patch_dir("new");
        let timeline = Timeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        assert_eq!(timeline.len(), 0);
        Ok(())
    }

    #[test]
    fn push() -> Result<(), TimelineError> {
        let patch_dir = patch_dir("push");
        let patch = Patch::from_data(&[2]);
        let mut timeline = Timeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        Ok(())
    }

    #[test]
    fn pop() -> Result<(), TimelineError> {
        let patch_dir = patch_dir("pop");
        let patch = Patch::from_data(&[2]);
        let mut timeline = Timeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        timeline.pop()?;
        assert!(timeline.is_empty());
        Ok(())
    }

    #[test]
    fn truncate() -> Result<(), TimelineError> {
        let patch_dir = patch_dir("truncate");
        let patch = Patch::from_data(&[2]);
        let mut timeline = Timeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        timeline.trunicate(0)?;
        assert!(timeline.is_empty());
        Ok(())
    }

    #[test]
    fn get() -> Result<(), TimelineError> {
        let patch_dir = patch_dir("get");
        let patch = Patch::from_data(&[2]);
        let mut timeline = Timeline::new(&patch_dir)?;
        assert!(timeline.is_empty());
        timeline.push(&patch)?;
        assert_eq!(timeline.len(), 1);
        let gotten_patch = timeline.get(0);
        assert!(gotten_patch.is_some());
        let gotten_patch = gotten_patch.unwrap()?;
        assert_eq!(gotten_patch, patch);
        Ok(())
    }
}
