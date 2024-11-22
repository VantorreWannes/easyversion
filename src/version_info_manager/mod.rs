use label::Label;
use serde::{Deserialize, Serialize};
use version_identifier::VersionIdentifier;
use version_info::VersionInfo;

pub mod label;
pub mod version_identifier;
pub mod version_info;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum VersionInfoManagerError {
    DuplicateLabel(Label),
}

impl std::fmt::Display for VersionInfoManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateLabel(label) => write!(f, "Duplicate label: {}", label),
        }
    }
}

impl std::error::Error for VersionInfoManagerError {}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct VersionInfoManager {
    versions: Vec<VersionInfo>,
}

impl VersionInfoManager {
    pub fn new() -> Self {
        Self { versions: vec![] }
    }

    pub fn versions(&self) -> &[VersionInfo] {
        &self.versions
    }

    pub fn get(&self, version_identifier: &VersionIdentifier) -> Option<&VersionInfo> {
        match version_identifier {
            VersionIdentifier::Index(index) => return self.versions.get(*index),
            VersionIdentifier::Label(label) => {
                return self.versions.iter().find(|v| v.label() == Some(label))
            }
        }
    }

    pub fn get_mut(&mut self, version_identifier: &VersionIdentifier) -> Option<&mut VersionInfo> {
        match version_identifier {
            VersionIdentifier::Index(index) => return self.versions.get_mut(*index),
            VersionIdentifier::Label(label) => {
                return self.versions.iter_mut().find(|v| v.label() == Some(label))
            }
        }
    }

    pub fn contains_label(&self, label: &Label) -> bool {
        self.versions.iter().any(|v| v.label() == Some(label))
    }

    pub fn set_label(
        &mut self,
        version_identifier: &VersionIdentifier,
        label: &Label,
    ) -> Result<(), VersionInfoManagerError> {
        if self.contains_label(label) {
            return Err(VersionInfoManagerError::DuplicateLabel(label.clone()));
        }
        if let Some(version) = self.get_mut(version_identifier) {
            version.set_label(label.clone());
        }
        Ok(())
    }

    pub fn add_version(&mut self) {
        self.versions.push(VersionInfo::new(self.versions.len()));
    }

    pub fn remove_version(&mut self, version_identifier: &VersionIdentifier) {
        if let Some(version_info) = self.get(version_identifier) {
            let index = version_info.index();
            self.versions.truncate(index);
        }
    }

    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.version_count() == 0
    }

    pub fn latest_version_index(&self) -> Option<usize> {
        match self.version_count() {
            0 => None,
            len => Some(len - 1),
        }
    }

    pub fn clear(&mut self) {
        self.versions.clear();
    }

    pub fn fork(&self) -> Self {
        let mut new_instance = self.clone();
        new_instance.clear();
        new_instance.add_version();
        new_instance
    }
}

#[cfg(test)]
mod version_info_manager_tests {
    use super::*;

    #[test]
    fn test_version_count() {
        let mut version_info_manager = VersionInfoManager::new();
        assert_eq!(version_info_manager.version_count(), 0);
        version_info_manager.add_version();
        assert_eq!(version_info_manager.version_count(), 1);
        version_info_manager.add_version();
        assert_eq!(version_info_manager.version_count(), 2);
    }

    #[test]
    fn test_latest_version_index() {
        let mut version_info_manager = VersionInfoManager::new();
        assert_eq!(version_info_manager.latest_version_index(), None);
        version_info_manager.add_version();
        assert_eq!(version_info_manager.latest_version_index(), Some(0));
        version_info_manager.add_version();
        assert_eq!(version_info_manager.latest_version_index(), Some(1));
    }

    #[test]
    fn test_clear() {
        let mut version_info_manager = VersionInfoManager::new();
        version_info_manager.add_version();
        version_info_manager.add_version();
        version_info_manager.clear();
        assert_eq!(version_info_manager.version_count(), 0);
    }

    #[test]
    fn test_fork() {
        let version_info_manager = VersionInfoManager::new();
        let forked_version_info_manager = version_info_manager.fork();
        assert_eq!(forked_version_info_manager.version_count(), 1);
    }

    #[test]
    fn test_set_label() {
        let mut version_info_manager = VersionInfoManager::new();
        version_info_manager.add_version();
        let label = Label::new("label").unwrap();
        assert!(version_info_manager
            .set_label(&VersionIdentifier::Index(0), &label)
            .is_ok());
        assert!(version_info_manager
            .set_label(&VersionIdentifier::Index(0), &label)
            .is_err());
    }
}
