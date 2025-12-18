use std::{
    fs,
    hash::Hash,
    path::{Path, PathBuf},
};

use gxhash::GxBuildHasher;
use serde::{Deserialize, Serialize};
use std::hash::BuildHasher;

use crate::file::id::FileId;

pub(crate) trait Config
where
    for<'a> Self: Hash + Serialize + Deserialize<'a>,
{
    fn directory_id(directory: &Path) -> FileId {
        FileId::new(GxBuildHasher::default().hash_one(directory))
    }

    fn config_file_path(config_directory: &Path, file_id: FileId) -> PathBuf {
        config_directory.join(format!("{}.json", file_id))
    }

    fn to_file(&self, config_directory: &Path, file_id: FileId) -> anyhow::Result<()> {
        fs::create_dir_all(config_directory)?;
        let file_path = Self::config_file_path(config_directory, file_id);
        log::trace!("Saving config to file: {:?}", file_path);
        let json_data = serde_json::to_string(&self)?;

        fs::write(&file_path, json_data)?;
        log::trace!("Config saved successfully to: {:?}", file_path);

        Ok(())
    }

    fn from_file(config_directory: &Path, file_id: FileId) -> anyhow::Result<Option<Self>> {
        let file_path = Self::config_file_path(config_directory, file_id);

        if !file_path.try_exists()? {
            log::trace!("Config file does not exist: {:?}", file_path);
            return Ok(None);
        }

        log::trace!("Loading config from file: {:?}", file_path);
        let json_data = fs::read(&file_path)?;
        let config = serde_json::from_slice(&json_data)?;
        log::trace!("Config loaded successfully from: {:?}", &file_path);
        Ok(Some(config))
    }
}
