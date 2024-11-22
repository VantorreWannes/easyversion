use serde::{Deserialize, Serialize};

use super::label::Label;

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