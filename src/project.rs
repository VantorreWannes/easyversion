use std::{
    fs, io,
    path::{Path, PathBuf},
};

use names::Generator;
use serde::{Deserialize, Serialize};

use crate::tracked::{
    file::TrackedFile,
    folder::{TrackedFolder, TrackedItem},
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    name: String,
    description: Option<String>,
}

impl VersionInfo {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(|description| description.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        let mut generator = Generator::default();
        let name = generator.next().expect("RNG is available");
        Self {
            name: format!("Version Identifier: {}.", name),
            description: None,
        }
    }
}

impl From<&str> for VersionInfo {
    fn from(value: &str) -> Self {
        Self::new(value, None)
    }
}

#[cfg(test)]
mod version_info_tests {
    use super::*;

    #[test]
    fn test_default() {
        let _ = VersionInfo::default();
    }

    #[test]
    fn test_new() {
        let name = "Test name";
        let description = "Test description";
        let version_info = VersionInfo::new(name, Some(description));
        assert_eq!(version_info.name(), name);
        assert_eq!(version_info.description(), Some(description));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Project {
    patch_dir: PathBuf,
    tracked_item: TrackedItem,
    version_infos: Vec<VersionInfo>,
}

impl Project {
    const DEFAULT_PATCH_DIR: &'static str = "test-data/project/patches";

    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::with_patch_dir(path, Self::DEFAULT_PATCH_DIR)
    }

    pub fn with_patch_dir(path: impl AsRef<Path>, patch_dir: impl AsRef<Path>) -> io::Result<Self> {
        let project;
        if path.as_ref().is_file() {
            let mut tracked_file = TrackedFile::new(path.as_ref(), patch_dir.as_ref());
            tracked_file.save()?;
            project = Self {
                tracked_item: tracked_file.into(),
                version_infos: vec![],
                patch_dir: patch_dir.as_ref().to_path_buf(),
            };
        } else if path.as_ref().is_dir() {
            let mut tracked_folder = TrackedFolder::new(path.as_ref(), patch_dir.as_ref())?;
            tracked_folder.save()?;
            project = Self {
                tracked_item: tracked_folder.into(),
                version_infos: vec![],
                patch_dir: patch_dir.as_ref().to_path_buf(),
            };
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is neither a file nor a directory",
            ));
        }
        Ok(project)
    }

    pub fn patch_dir(&self) -> &Path {
        &self.patch_dir
    }

    pub fn save(&mut self, version_info: impl Into<VersionInfo>) -> io::Result<()> {
        self.tracked_item.save()?;
        self.version_infos.push(version_info.into());
        Ok(())
    }

    pub fn versions(&self) -> &[VersionInfo] {
        &self.version_infos
    }

    pub fn load(&mut self, version: usize) -> io::Result<()> {
        self.tracked_item.load(version)
    }

    pub fn write_to(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let serialized = ron::to_string(self).expect("serializing should succeed");
        fs::write(path, serialized)
    }

    pub fn open_from(path: impl AsRef<Path>) -> io::Result<Self> {
        let serialized = fs::read_to_string(path)?;
        let project = ron::from_str(&serialized).expect("deserializing should succeed");
        Ok(project)
    }
}

#[cfg(test)]
mod project_tests {
    use super::*;

    #[test]
    fn test_new() {
        let project = Project::new("test-data/project/items");
        assert!(project.is_ok());
    }

    #[test]
    fn test_with_patch_dir() {
        let project =
            Project::with_patch_dir("test-data/project/items", "test-data/project/patches");
        assert!(project.is_ok());
    }

    #[test]
    fn test_save() -> io::Result<()> {
        let mut project = Project::new("test-data/project/items").unwrap();
        project.save(VersionInfo::default())
    }

    #[test]
    fn test_load() -> io::Result<()> {
        let mut project = Project::new("test-data/project/items").unwrap();
        project.save(VersionInfo::default())?;
        project.load(0)
    }

    #[test]
    fn test_write_to() -> io::Result<()> {
        let mut project = Project::new("test-data/project/items").unwrap();
        project.save(VersionInfo::default())?;
        project.write_to("test-data/project/patches/ezproject.ron")
    }
}
