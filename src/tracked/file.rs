use std::{
    error::Error,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    hash,
    patches::{
        patch::Patch,
        patch_timeline::{PatchTimeline, PatchTimelineError},
    },
};

use super::{Version, VersionError};

#[derive(Debug)]
pub enum TrackedFileError {
    PatchTimelineError(PatchTimelineError),
    FileDoesntExist,
}

impl Display for TrackedFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackedFileError::PatchTimelineError(err) => write!(f, "{}", err),
            TrackedFileError::FileDoesntExist => write!(f, "File at path doesn't exist"),
        }
    }
}

impl Error for TrackedFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TrackedFileError::PatchTimelineError(err) => Some(err),
            TrackedFileError::FileDoesntExist => None,
        }
    }
}

impl From<PatchTimelineError> for TrackedFileError {
    fn from(err: PatchTimelineError) -> Self {
        Self::PatchTimelineError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct TrackedFile {
    path: PathBuf,
    patch_timeline: PatchTimeline,
}

impl TrackedFile {
    pub fn new(
        file_path: impl AsRef<Path>,
        patch_dir: impl AsRef<Path>,
    ) -> Result<Self, TrackedFileError> {
        let path = file_path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(TrackedFileError::FileDoesntExist);
        }
        let patch_dir = patch_dir.as_ref().join(hash(&path).to_string());
        let patch_timeline = PatchTimeline::new(patch_dir)?;
        Ok(Self {
            path,
            patch_timeline,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn patch_timeline(&self) -> &PatchTimeline {
        &self.patch_timeline
    }

    pub fn apply(&self, index: usize) -> Result<Vec<u8>, VersionError> {
        if self.is_empty() {
            return Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable,
            ));
        }
        let mut source = vec![];
        for i in 0..=index {
            let patch = self.patch_timeline.get(i)?;
            source = patch
                .apply(source.as_slice())
                .map_err(PatchTimelineError::from)?;
        }
        Ok(source)
    }
}

impl Version for TrackedFile {
    fn commit(&mut self) -> Result<(), super::VersionError> {
        let source = match self.latest_version_index() {
            Some(index) => self.apply(index)?,
            None => vec![],
        };
        let target = fs::read(&self.path).map_err(PatchTimelineError::from)?;
        let patch = Patch::new(&source, &target).map_err(PatchTimelineError::from)?;
        self.patch_timeline.push(&patch)?;
        Ok(())
    }

    fn load_version(&self, index: usize) -> Result<(), super::VersionError> {
        let content = self.apply(index)?;
        fs::write(&self.path, content).map_err(PatchTimelineError::from)?;
        Ok(())
    }

    fn delete_version(&mut self, index: usize) -> Result<(), super::VersionError> {
        match self.latest_version_index() {
            Some(latest_index) => {
                for _ in index..=latest_index {
                    self.patch_timeline.pop()?;
                }
                Ok(())
            }
            None => Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable,
            )),
        }
    }

    fn version_count(&self) -> usize {
        self.patch_timeline.len()
    }
}

#[cfg(test)]
mod tracked_file_tests {
    use fs::File;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn new() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        let tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        assert_eq!(tracked_file.path(), &file_path);
        assert_eq!(tracked_file.version_count(), 0);
    }

    #[test]
    fn commit() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        fs::write(&file_path, "hello world").unwrap();
        let mut tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        tracked_file.commit().unwrap();
        assert_eq!(tracked_file.version_count(), 1);
    }

    #[test]
    fn apply_no_versions_available() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        let tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        let result = tracked_file.apply(0);
        assert!(matches!(
            result,
            Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable
            ))
        ));
    }

    #[test]
    fn apply() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        fs::write(&file_path, b"hello world").unwrap();
        let mut tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        tracked_file.commit().unwrap();
        let source = tracked_file.apply(0).unwrap();
        assert_eq!(&source, b"hello world");
    }

    #[test]
    fn load_version() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        fs::write(&file_path, b"hello world").unwrap();
        let mut tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        tracked_file.commit().unwrap();
        tracked_file.load_version(0).unwrap();
        let content = fs::read(&file_path).unwrap();
        assert_eq!(&content, b"hello world");
    }

    #[test]
    fn load_version_no_versions_available() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        let tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        let result = tracked_file.load_version(0);
        assert!(matches!(
            result,
            Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable
            ))
        ));
    }

    #[test]
    fn delete_version() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        fs::write(&file_path, b"hello world").unwrap();
        let mut tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        tracked_file.commit().unwrap();
        tracked_file.delete_version(0).unwrap();
        let result = tracked_file.load_version(0);
        assert!(matches!(
            result,
            Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable
            ))
        ));
    }

    #[test]
    fn delete_version_no_versions_available() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        let mut tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        let result = tracked_file.delete_version(0);
        assert!(matches!(
            result,
            Err(VersionError::PatchTimelineError(
                PatchTimelineError::NoVersionsAvailable
            ))
        ));
    }

    #[test]
    fn version_count() {
        let dir = TempDir::new("easyversion").unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        let tracked_file = TrackedFile::new(&file_path, dir.path()).unwrap();
        assert_eq!(tracked_file.version_count(), 0);
    }
}
