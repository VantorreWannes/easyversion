use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use log;
use serde::{Deserialize, Serialize};

use crate::{config::Config, file::storage::FileStorage, project::workspace::Workspace};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct EasyVersion {
    config_directory: PathBuf,
    data_directory: PathBuf,
}

impl EasyVersion {
    const STORAGE_DIRECTORY_NAME: &str = "files";
    const STORAGE_CONFIG_DIRECTORY_NAME: &str = "filestores";
    const WORKSPACE_CONFIGS_DIRECTORY_NAME: &str = "workspaces";

    pub fn new(config_directory: &Path, data_directory: &Path) -> Self {
        Self {
            config_directory: config_directory.to_path_buf(),
            data_directory: data_directory.to_path_buf(),
        }
    }

    fn storage_directory(&self) -> PathBuf {
        self.data_directory.join(Self::STORAGE_DIRECTORY_NAME)
    }

    fn storage_config_directory(&self) -> PathBuf {
        self.config_directory
            .join(Self::STORAGE_CONFIG_DIRECTORY_NAME)
    }

    fn workspace_config_directory(&self) -> PathBuf {
        self.config_directory
            .join(Self::WORKSPACE_CONFIGS_DIRECTORY_NAME)
    }

    fn open_storage(&self) -> anyhow::Result<FileStorage> {
        log::trace!("Opening storage for directory: {:?}", self.data_directory);
        let storage_directory = self.storage_directory();
        let file_id = FileStorage::directory_id(&storage_directory);
        let storage_config_directory = self.storage_config_directory();
        match FileStorage::from_file(&storage_config_directory, file_id)? {
            Some(file_storage) => {
                log::debug!("Loaded existing storage configuration");
                Ok(file_storage)
            }
            None => {
                log::debug!("Creating new storage in: {:?}", storage_directory);
                Ok(FileStorage::new(&storage_directory)?)
            }
        }
    }

    fn open_workspace(&self, workspace_directory: &Path) -> anyhow::Result<Option<Workspace>> {
        log::debug!("Opening workspace for directory: {:?}", workspace_directory);
        let file_id = Workspace::directory_id(workspace_directory);
        let workspace_config_directory = self.workspace_config_directory();
        let workspace = Workspace::from_file(&workspace_config_directory, file_id)?;
        match &workspace {
            Some(_) => log::debug!("Loaded existing workspace configuration"),
            None => log::debug!("No existing workspace configuration found"),
        }
        Ok(workspace)
    }

    fn save_storage(&self, storage: &FileStorage) -> anyhow::Result<()> {
        log::trace!("Saving storage configuration");
        let file_id = FileStorage::directory_id(storage.directory());
        let config_directory = self.storage_config_directory();
        storage.to_file(&config_directory, file_id)?;
        log::debug!("Storage configuration saved");
        Ok(())
    }

    fn remove_workspace(&self, workspace_directory: &Path) -> anyhow::Result<()> {
        log::debug!(
            "Removing workspace for directory: {:?}",
            workspace_directory
        );
        let file_id = Workspace::directory_id(workspace_directory);
        let workspace_config_directory = self.workspace_config_directory();
        let workspace_config_path =
            Workspace::config_file_path(&workspace_config_directory, file_id);
        fs::remove_file(workspace_config_path)?;
        Ok(())
    }

    fn save_workspace(&self, workspace: &Workspace) -> anyhow::Result<()> {
        log::trace!(
            "Saving workspace configuration for: {:?}",
            workspace.directory()
        );
        let file_id = Workspace::directory_id(workspace.directory());
        let config_directory = self.workspace_config_directory();
        workspace.to_file(&config_directory, file_id)?;
        log::debug!("Workspace configuration saved");
        Ok(())
    }

    pub fn save(&self, workspace_directory: &Path, comment: Option<&str>) -> anyhow::Result<()> {
        log::info!(
            "Starting save operation for workspace: {:?}",
            workspace_directory
        );
        let mut storage = self.open_storage().context("Failed to open storage")?;
        let mut workspace = self
            .open_workspace(workspace_directory)
            .context("Failed to open workspace")?
            .unwrap_or_else(|| {
                log::debug!("Creating new workspace for: {:?}", workspace_directory);
                Workspace::new(workspace_directory).unwrap()
            });

        workspace
            .save_version(&mut storage, comment)
            .context("Failed to save version to workspace")?;

        self.save_storage(&storage)
            .context("Failed to save storage configuration")?;
        self.save_workspace(&workspace)
            .context("Failed to save workspace configuration")?;

        let version_number = workspace.versions().len();
        println!(
            "Saved version {} for {:?}",
            version_number, &workspace_directory
        );

        Ok(())
    }

    pub fn list(&self, workspace_directory: &Path) -> anyhow::Result<()> {
        log::info!("Listing versions for workspace: {:?}", workspace_directory);
        let workspace = self
            .open_workspace(workspace_directory)?
            .unwrap_or_else(|| {
                log::debug!("No workspace found, creating temporary one");
                Workspace::new(workspace_directory).unwrap()
            });

        let versions = workspace.versions();

        if versions.is_empty() {
            println!("No versions saved yet.");
            return Ok(());
        }

        println!("Saved versions ({}):", versions.len());
        for (index, version) in versions.iter().enumerate() {
            let comment = version
                .comment()
                .clone()
                .unwrap_or_else(|| "(no comment)".to_string());
            println!("{:>3}. {}", index + 1, comment);
        }
        Ok(())
    }

    pub fn split(
        &self,
        source_workspace_directory: &Path,
        target_workspace_directory: &Path,
        version_number: Option<usize>,
        may_overwrite: bool,
    ) -> anyhow::Result<()> {
        log::info!(
            "Starting split operation from {:?} to {:?}",
            source_workspace_directory,
            target_workspace_directory
        );
        let mut storage = self.open_storage().context("Failed to open storage")?;
        let source_workspace = self
            .open_workspace(source_workspace_directory)
            .context("Failed to open source workspace")?
            .unwrap_or_else(|| {
                log::debug!("Creating source workspace");
                Workspace::new(source_workspace_directory).unwrap()
            });

        if !may_overwrite
            && target_workspace_directory
                .try_exists()
                .context("Failed to check if target directory exists")?
        {
            log::error!(
                "Target directory exists and overwrite not allowed: {:?}",
                target_workspace_directory
            );
            anyhow::bail!(
                "Target directory exists and overwrite not allowed: {:?}",
                target_workspace_directory
            );
        }

        if let Some(mut target_workspace) = self
            .open_workspace(target_workspace_directory)
            .context("Failed to check existing target workspace")?
        {
            log::debug!("Cleaning up existing target workspace");
            target_workspace
                .clean_up(&mut storage)
                .context("Failed to clean up existing target workspace")?;
        }

        let target_workspace = source_workspace
            .restore_version_to(&mut storage, target_workspace_directory, version_number)
            .context("Failed to restore version to target directory")?;

        self.save_storage(&storage)
            .context("Failed to save storage configuration")?;
        self.save_workspace(&target_workspace)
            .context("Failed to save target workspace configuration")?;

        println!(
            "Created split at {:?} with {} version(s).",
            target_workspace_directory,
            version_number
                .map(|n| n.to_string())
                .unwrap_or("no".to_owned())
        );
        Ok(())
    }

    pub fn clean(&self, workspace_directory: &Path) -> anyhow::Result<()> {
        log::info!(
            "Starting clean operation for workspace: {:?}",
            workspace_directory
        );
        let mut storage = self.open_storage().context("Failed to open storage")?;

        if let Some(mut workspace) = self
            .open_workspace(workspace_directory)
            .context("Failed to open workspace")?
        {
            log::debug!(
                "Cleaning up workspace with {} versions",
                workspace.versions().len()
            );
            workspace
                .clean_up(&mut storage)
                .context("Failed to clean up workspace")?;
            self.save_storage(&storage)
                .context("Failed to save storage configuration")?;
            self.remove_workspace(workspace_directory)?;
            println!("Cleaned EasyVersion data for {:?}", workspace_directory);
        } else {
            log::debug!("No workspace found to clean");
            println!(
                "Nothing to clean. No versions found for {:?}",
                workspace_directory
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const SMALL_DATA: [u8; 1] = [1];

    #[test]
    fn test_new() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();

        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        assert_eq!(easy_version.config_directory, temp_config_dir.path());
        assert_eq!(easy_version.data_directory, temp_data_dir.path());
    }

    #[test]
    fn test_save_creates_version() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let file_path = temp_workspace_dir.path().join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();

        let result = easy_version.save(temp_workspace_dir.path(), Some("test save"));
        assert!(result.is_ok());

        let workspace_config_dir = temp_config_dir.path().join("workspaces");
        assert!(workspace_config_dir.exists());
    }

    #[test]
    fn test_save_without_comment() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let result = easy_version.save(temp_workspace_dir.path(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_no_versions() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let result = easy_version.list(temp_workspace_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_versions() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let file_path = temp_workspace_dir.path().join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();

        easy_version
            .save(temp_workspace_dir.path(), Some("first"))
            .unwrap();
        fs::write(&file_path, [2]).unwrap();
        easy_version
            .save(temp_workspace_dir.path(), Some("second"))
            .unwrap();

        let result = easy_version.list(temp_workspace_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_split_full() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_source_dir = TempDir::new().unwrap();
        let temp_target_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let file_path = temp_source_dir.path().join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();
        easy_version
            .save(temp_source_dir.path(), Some("v1"))
            .unwrap();

        let result = easy_version.split(temp_source_dir.path(), temp_target_dir.path(), None, true);
        assert!(result.is_ok());

        assert!(temp_target_dir.path().join("test.txt").exists());
    }

    #[test]
    fn test_split_partial() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_source_dir = TempDir::new().unwrap();
        let temp_target_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let file_path = temp_source_dir.path().join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();
        easy_version
            .save(temp_source_dir.path(), Some("v1"))
            .unwrap();
        fs::write(&file_path, [2]).unwrap();
        easy_version
            .save(temp_source_dir.path(), Some("v2"))
            .unwrap();

        let result = easy_version.split(
            temp_source_dir.path(),
            temp_target_dir.path(),
            Some(0),
            true,
        );
        assert!(result.is_ok());

        assert!(temp_target_dir.path().join("test.txt").exists());
        assert_eq!(
            fs::read(temp_target_dir.path().join("test.txt")).unwrap(),
            SMALL_DATA
        );
    }

    #[test]
    fn test_split_directory_exists_error() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_source_dir = TempDir::new().unwrap();
        let temp_target_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        fs::create_dir_all(temp_target_dir.path()).unwrap();

        let result =
            easy_version.split(temp_source_dir.path(), temp_target_dir.path(), None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_existing_workspace() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let file_path = temp_workspace_dir.path().join("test.txt");
        fs::write(&file_path, SMALL_DATA).unwrap();
        easy_version
            .save(temp_workspace_dir.path(), Some("v1"))
            .unwrap();

        let result = easy_version.clean(temp_workspace_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_no_workspace() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_data_dir = TempDir::new().unwrap();
        let temp_workspace_dir = TempDir::new().unwrap();
        let easy_version = EasyVersion::new(temp_config_dir.path(), temp_data_dir.path());

        let result = easy_version.clean(temp_workspace_dir.path());
        assert!(result.is_ok());
    }
}
