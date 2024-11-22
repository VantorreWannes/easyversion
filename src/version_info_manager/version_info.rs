use serde::{Deserialize, Serialize};

use super::label::Label;

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
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

    pub(super) fn set_label(&mut self, label: Label) {
        self.label = Some(label);
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_owned());
    }

    pub fn clear_label(&mut self) {
        self.label = None;
    }

    pub fn clear_message(&mut self) {
        self.message = None;
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

}

impl PartialOrd for VersionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

