use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use log;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::{file::storage::FileStorage, project::version::Version};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Workspace {
    directory: PathBuf,
    versions: Vec<Version>,
}

impl Workspace {
    pub fn new(directory: &Path) -> anyhow::Result<Self> {
        log::debug!("Creating new workspace in directory: {:?}", directory);
        fs::create_dir_all(directory)?;
        log::debug!("Workspace created successfully in: {:?}", directory);
        Ok(Self {
            directory: directory.to_path_buf(),
            versions: Default::default(),
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn versions(&self) -> &[Version] {
        &self.versions
    }

    pub fn save_version(
        &mut self,
        storage: &mut FileStorage,
        comment: Option<&str>,
    ) -> anyhow::Result<()> {
        log::debug!("Saving version for workspace: {:?}", self.directory);
        let mut version = Version::new(comment);

        version.store_directory(storage, &self.directory)?;

        self.versions.push(version);
        let version_count = self.versions.len();
        log::trace!("Version {} saved successfully", version_count);

        Ok(())
    }

    pub fn restore_version_to(
        &self,
        storage: &mut FileStorage,
        target_directory: &Path,
        version_number: Option<usize>,
    ) -> anyhow::Result<Workspace> {
        log::debug!(
            "Restoring workspace to: {:?}, version: {:?}",
            target_directory,
            version_number
        );
        let versions = match version_number {
            Some(version_number) => {
                if version_number >= self.versions.len() {
                    log::error!(
                        "Requested version {} exceeds available versions {}",
                        version_number,
                        self.versions.len()
                    );
                    anyhow::bail!(
                        "Versions exceeded: There are only {} versions stored right now",
                        self.versions.len()
                    );
                }
                log::debug!("Restoring up to version {}", version_number);
                self.versions[0..=version_number].to_vec()
            }
            None => {
                log::debug!("Restoring all {} versions", self.versions.len());
                self.versions.clone()
            }
        };

        for version in &versions {
            version.increment_references(storage)?;
        }

        if let Some(last_version) = versions.last() {
            last_version.restore_directory_to(storage, target_directory)?;
        } else {
            log::debug!(
                "No versions to restore, creating temporary version from current directory"
            );
            let mut temp_version = Version::new(None);
            temp_version.store_directory(storage, &self.directory)?;
            temp_version.restore_directory_to(storage, target_directory)?;
            temp_version.clean_up(storage)?;
        }

        log::info!("Workspace restored successfully to: {:?}", target_directory);
        Ok(Workspace {
            directory: target_directory.to_path_buf(),
            versions,
        })
    }

    pub fn clean_up(&mut self, storage: &mut FileStorage) -> anyhow::Result<()> {
        log::debug!(
            "Cleaning up workspace with {} versions",
            self.versions.len()
        );
        for version in &self.versions {
            version.clean_up(storage)?;
        }
        self.versions.clear();
        log::debug!("Workspace cleanup completed");

        Ok(())
    }
}

impl Hash for Workspace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.directory.hash(state);
    }
}

impl Config for Workspace {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SMALL_DATA: [u8; 1] = [1];

    #[test]
    fn test_new_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().join("workspace");

        let workspace = Workspace::new(&workspace_path).unwrap();

        assert_eq!(workspace.directory(), workspace_path);
        assert!(workspace.versions().is_empty());
        assert!(workspace_path.exists() && workspace_path.is_dir());
    }

    #[test]
    fn test_directory_and_versions_getters() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().join("workspace");
        let mut workspace = Workspace::new(&workspace_path).unwrap();
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();

        let file_path = workspace_path.join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();

        workspace
            .save_version(&mut storage, Some("test version"))
            .unwrap();

        assert_eq!(workspace.directory(), workspace_path);
        assert_eq!(workspace.versions().len(), 1);
        assert_eq!(
            workspace.versions()[0].comment(),
            &Some("test version".to_string())
        );
    }

    #[test]
    fn test_save_adds_version() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let mut workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        let file_path = temp_workspace_dir.path().join("file.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();

        workspace
            .save_version(&mut storage, Some("first save"))
            .unwrap();

        assert_eq!(workspace.versions().len(), 1);
        assert_eq!(
            workspace.versions()[0].comment(),
            &Some("first save".to_string())
        );
        assert!(
            workspace.versions()[0]
                .files()
                .contains_key(&PathBuf::from("file.txt"))
        );
    }

    #[test]
    fn test_save_with_no_comment() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let mut workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        workspace.save_version(&mut storage, None).unwrap();

        assert_eq!(workspace.versions().len(), 1);
        assert_eq!(workspace.versions()[0].comment(), &None);
    }

    #[test]
    fn test_restore_to_with_none_copies_all_versions() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let mut workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        fs::write(temp_workspace_dir.path().join("file1.txt"), SMALL_DATA).unwrap();
        workspace.save_version(&mut storage, Some("v1")).unwrap();
        fs::write(temp_workspace_dir.path().join("file2.txt"), SMALL_DATA).unwrap();
        workspace.save_version(&mut storage, Some("v2")).unwrap();

        let temp_restore_to_dir = TempDir::new().unwrap();
        let restore_to_workspace = workspace
            .restore_version_to(&mut storage, temp_restore_to_dir.path(), None)
            .unwrap();

        assert_eq!(restore_to_workspace.versions().len(), 2);
        assert_eq!(
            restore_to_workspace.versions()[0].comment(),
            &Some("v1".to_string())
        );
        assert_eq!(
            restore_to_workspace.versions()[1].comment(),
            &Some("v2".to_string())
        );
        assert_eq!(restore_to_workspace.directory(), temp_restore_to_dir.path());
        assert!(temp_restore_to_dir.path().join("file1.txt").exists());
        assert!(temp_restore_to_dir.path().join("file2.txt").exists());
    }

    #[test]
    fn test_restore_to_with_some_index_copies_up_to_index() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let mut workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        fs::write(temp_workspace_dir.path().join("file1.txt"), SMALL_DATA).unwrap();
        workspace.save_version(&mut storage, Some("v1")).unwrap();
        fs::write(temp_workspace_dir.path().join("file2.txt"), SMALL_DATA).unwrap();
        workspace.save_version(&mut storage, Some("v2")).unwrap();

        let temp_restore_to_dir = TempDir::new().unwrap();
        let restore_to_workspace = workspace
            .restore_version_to(&mut storage, temp_restore_to_dir.path(), Some(0))
            .unwrap();

        assert_eq!(restore_to_workspace.versions().len(), 1);
        assert_eq!(
            restore_to_workspace.versions()[0].comment(),
            &Some("v1".to_string())
        );
        assert_eq!(restore_to_workspace.directory(), temp_restore_to_dir.path());
        assert!(temp_restore_to_dir.path().join("file1.txt").exists());
        assert!(!temp_restore_to_dir.path().join("file2.txt").exists());
    }

    #[test]
    fn test_restore_to_with_version_number_too_large_errors() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let mut workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        workspace.save_version(&mut storage, Some("v1")).unwrap();

        let temp_restore_to_dir = TempDir::new().unwrap();
        let result =
            workspace.restore_version_to(&mut storage, temp_restore_to_dir.path(), Some(1));

        assert!(result.is_err());
    }

    #[test]
    fn test_restore_to_on_empty_workspace_restores_target_directory() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_workspace_dir.path()).unwrap();
        let temp_restore_to_dir = TempDir::new().unwrap();
        fs::write(temp_restore_to_dir.path().join("existing.txt"), SMALL_DATA).unwrap();

        let restore_to_workspace = workspace
            .restore_version_to(&mut storage, temp_restore_to_dir.path(), None)
            .unwrap();

        assert!(restore_to_workspace.versions().is_empty());
        assert_eq!(restore_to_workspace.directory(), temp_restore_to_dir.path());
    }

    #[test]
    fn test_cleanup_decrements_references() {
        let temp_storage_dir = TempDir::new().unwrap();
        let mut storage = FileStorage::new(temp_storage_dir.path()).unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let mut workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        let file_path = temp_workspace_dir.path().join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();
        workspace.save_version(&mut storage, Some("v1")).unwrap();
        let file_id = workspace.versions()[0]
            .files()
            .values()
            .next()
            .cloned()
            .unwrap();

        assert!(storage.reference_count(file_id).unwrap() == 1);

        workspace.clean_up(&mut storage).unwrap();
    }

    #[test]
    fn test_serialization_roundtrip() {
        let temp_workspace_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_workspace_dir.path()).unwrap();

        let serialized = serde_json::to_string(&workspace).unwrap();
        let deserialized: Workspace = serde_json::from_str(&serialized).unwrap();

        assert_eq!(workspace, deserialized);
    }
}
