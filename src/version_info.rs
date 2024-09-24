use std::{fmt::Display, hash::Hash};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LabelError {
    InvalidLabel(String),
}

impl Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLabel(label) => write!(f, "Invalid label: {}", label),
        }
    }
}

impl std::error::Error for LabelError {}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn new(name: &str) -> Result<Self, LabelError> {
        if !Self::is_valid_name(name) {
            return Err(LabelError::InvalidLabel(name.to_string()));
        }
        Ok(Self {
            name: name.to_string(),
        })
    }

    fn is_valid_name(name: &str) -> bool {
        name.chars().all(|c| !c.is_whitespace())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod label_tests {

    use super::*;

    #[test]
    fn new() {
        assert!(Label::new("v1.0").is_ok());
        assert!(Label::new("foobar").is_ok());
        assert!(Label::new("foo-bar").is_ok());
        assert!(Label::new(" ").is_err());
        assert!(Label::new("foo bar").is_err());
    }

    #[test]
    fn is_valid_name() {
        assert!(Label::is_valid_name("v1.0"));
        assert!(Label::is_valid_name("foobar"));
        assert!(Label::is_valid_name("foo-bar"));
        assert!(!Label::is_valid_name(" "));
        assert!(!Label::is_valid_name("foo bar"));
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Hash)]
pub enum VersionIdentifier {
    Index(usize),
    Label(Label),
}

impl VersionIdentifier {
    pub fn from_label(label: Label) -> Self {
        Self::Label(label)
    }

    pub fn from_index(index: usize) -> Self {
        Self::Index(index)
    }

    pub fn index(&self) -> Option<usize> {
        match self {
            Self::Index(index) => Some(*index),
            _ => None,
        }
    }

    pub fn label(&self) -> Option<&Label> {
        match self {
            Self::Label(label) => Some(label),
            _ => None,
        }
    }
}

impl From<Label> for VersionIdentifier {
    fn from(label: Label) -> Self {
        Self::from_label(label)
    }
}

impl From<usize> for VersionIdentifier {
    fn from(index: usize) -> Self {
        Self::from_index(index)
    }
}

impl Display for VersionIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Index(index) => write!(f, "{}", index),
            Self::Label(label) => write!(f, "{}", label.name()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct VersionInfo {
    index: usize,
    label: Option<Label>,
    message: Option<String>,
}

impl VersionInfo {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            label: None,
            message: None,
        }
    }

    pub fn with_message(index: usize, message: &str) -> Self {
        Self {
            index,
            label: None,
            message: Some(message.to_owned()),
        }
    }
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn set_label(&mut self, label: Label) {
        self.label = Some(label);
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }

    pub fn clear_label(&mut self) {
        self.label = None;
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn identifier(&self) -> VersionIdentifier {
        if let Some(label) = &self.label {
            VersionIdentifier::from_label(label.clone())
        } else {
            VersionIdentifier::from_index(self.index)
        }
    }
}

impl PartialOrd for VersionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

#[cfg(test)]
mod version_info_tests {

    use super::*;

    #[test]
    fn new() {
        assert!(VersionInfo::new(0).label.is_none());
        assert!(VersionInfo::new(0).message.is_none());
        assert!(VersionInfo::new(1).label.is_none());
        assert!(VersionInfo::new(1).message.is_none());
        assert!(VersionInfo::new(2).label.is_none());
        assert!(VersionInfo::new(2).message.is_none());
    }

    #[test]
    fn with_message() {
        assert!(VersionInfo::with_message(0, "0").label.is_none());
        assert!(VersionInfo::with_message(0, "0")
            .message
            .is_some_and(|message| message == "0"));
        assert!(VersionInfo::with_message(1, "1").label.is_none());
        assert!(VersionInfo::with_message(1, "1")
            .message
            .is_some_and(|message| message == "1"));
        assert!(VersionInfo::with_message(2, "2").label.is_none());
        assert!(VersionInfo::with_message(2, "2")
            .message
            .is_some_and(|message| message == "2"));
    }

    #[test]
    fn identifier() {
        assert!(VersionInfo::new(0).identifier().label().is_none());
        assert!(VersionInfo::new(0)
            .identifier()
            .index()
            .is_some_and(|index| index == 0));
        assert!(VersionInfo::new(1).identifier().label().is_none());
        assert!(VersionInfo::new(1)
            .identifier()
            .index()
            .is_some_and(|index| index == 1));
        assert!(VersionInfo::new(2).identifier().label().is_none());
        assert!(VersionInfo::new(2)
            .identifier()
            .index()
            .is_some_and(|index| index == 2));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum VersionInfoManagerError {
    VersionIdentifierNotFound(VersionIdentifier),
    DuplicateLabel(Label),
    DuplicateVersionInfo(VersionInfo),
}

impl Display for VersionInfoManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionIdentifierNotFound(version_identifier) => {
                write!(f, "Version identifier not found: {:?}", version_identifier)
            }
            Self::DuplicateLabel(label) => {
                write!(f, "There is already a version with label: {:?}", label)
            }
            Self::DuplicateVersionInfo(version_info) => {
                write!(
                    f,
                    "There is already a version with info: {:?}",
                    version_info
                )
            }
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
        Self::from_versions(vec![])
    }

    pub fn from_versions(versions: Vec<VersionInfo>) -> Self {
        Self { versions }
    }

    pub fn versions(&self) -> &[VersionInfo] {
        &self.versions
    }

    pub fn get(&self, version_identifier: &VersionIdentifier) -> Option<&VersionInfo> {
        match version_identifier {
            VersionIdentifier::Index(index) => self
                .versions
                .iter()
                .find(|version| version.index() == *index),
            VersionIdentifier::Label(label) => self
                .versions
                .iter()
                .find(|version| version.label() == Some(label)),
        }
    }

    pub fn get_mut(&mut self, version_identifier: &VersionIdentifier) -> Option<&mut VersionInfo> {
        match version_identifier {
            VersionIdentifier::Index(index) => self
                .versions
                .iter_mut()
                .find(|version| version.index() == *index),
            VersionIdentifier::Label(label) => self
                .versions
                .iter_mut()
                .find(|version| version.label() == Some(label)),
        }
    }

    pub fn contains_label(&self, label: &Label) -> bool {
        self.versions
            .iter()
            .any(|version| version.label() == Some(label))
    }

    pub fn set_label(
        &mut self,
        version_identifier: &VersionIdentifier,
        label: Label,
    ) -> Result<(), VersionInfoManagerError> {
        if self.contains_label(&label) {
            Err(VersionInfoManagerError::VersionIdentifierNotFound(
                version_identifier.clone(),
            ))
        } else if let Some(version) = self.get_mut(version_identifier) {
            version.set_label(label);
            Ok(())
        } else {
            Err(VersionInfoManagerError::VersionIdentifierNotFound(
                version_identifier.clone(),
            ))
        }
    }

    pub fn add_version_info(&mut self) {
        let version_info = VersionInfo::new(self.versions.len());
        self.versions.push(version_info);
    }

    pub fn len(&self) -> usize {
        self.versions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }
}

#[cfg(test)]
mod version_info_manager_tests {

    use super::*;

    #[test]
    fn new() {
        assert!(VersionInfoManager::new().versions().is_empty());
    }

    #[test]
    fn versions() {
        let versions = vec![];
        assert!(VersionInfoManager::from_versions(versions.clone())
            .versions()
            .is_empty());
        let versions = vec![
            VersionInfo::new(0),
            VersionInfo::new(1),
            VersionInfo::new(2),
        ];
        assert_eq!(
            VersionInfoManager::from_versions(versions.clone()).versions(),
            &versions
        );
    }

    #[test]
    fn get() {
        let versions = vec![
            VersionInfo::new(0),
            VersionInfo::new(1),
            VersionInfo::new(2),
        ];
        let manager = VersionInfoManager::from_versions(versions.clone());
        assert!(manager.get(&VersionIdentifier::from_index(0)).is_some());
        assert!(manager.get(&VersionIdentifier::from_index(1)).is_some());
        assert!(manager.get(&VersionIdentifier::from_index(2)).is_some());
        assert!(manager
            .get(&VersionIdentifier::from_label(Label::new("foo").unwrap()))
            .is_none());
    }

    #[test]
    fn get_mut() {
        let versions = vec![
            VersionInfo::new(0),
            VersionInfo::new(1),
            VersionInfo::new(2),
        ];
        let mut manager = VersionInfoManager::from_versions(versions.clone());
        assert!(manager.get_mut(&VersionIdentifier::from_index(0)).is_some());
        assert!(manager.get_mut(&VersionIdentifier::from_index(1)).is_some());
        assert!(manager.get_mut(&VersionIdentifier::from_index(2)).is_some());
        assert!(manager
            .get_mut(&VersionIdentifier::from_label(Label::new("foo").unwrap()))
            .is_none());
    }

    #[test]
    fn contains_label() {
        let mut version = VersionInfo::new(2);
        version.set_label(Label::new("foo").unwrap());
        let versions = vec![VersionInfo::new(0), VersionInfo::new(1), version];
        let manager = VersionInfoManager::from_versions(versions.clone());
        assert!(manager.contains_label(&Label::new("foo").unwrap()));
        assert!(!manager.contains_label(&Label::new("1.0").unwrap()));
    }

    #[test]
    fn set_label() {
        let mut version = VersionInfo::new(2);
        version.set_label(Label::new("foo").unwrap());
        let versions = vec![VersionInfo::new(0), VersionInfo::new(1), version];
        let mut manager = VersionInfoManager::from_versions(versions.clone());
        assert!(manager
            .set_label(
                &VersionIdentifier::from_index(2),
                Label::new("bar").unwrap()
            )
            .is_ok());
        assert!(manager
            .set_label(
                &VersionIdentifier::from_index(2),
                Label::new("foo").unwrap()
            )
            .is_ok());
        assert!(manager
            .set_label(
                &VersionIdentifier::from_label(Label::new("bar").unwrap()),
                Label::new("baz").unwrap()
            )
            .is_err());
    }

    #[test]
    fn add_version_info() {
        let mut manager = VersionInfoManager::new();
        manager.add_version_info();
        assert_eq!(manager.versions().len(), 1);
    }
}
