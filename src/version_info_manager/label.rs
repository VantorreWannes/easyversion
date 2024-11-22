use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LabelError {
    ContainsWhitespace,
}

impl Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContainsWhitespace => write!(f, "Label cannot contain whitespace"),
        }
    }
}

impl std::error::Error for LabelError {}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn new(name: &str) -> Result<Label, LabelError> {
        if Self::is_valid_name(name) {
            Ok(Label {
                name: name.to_string(),
            })
        } else {
            Err(LabelError::ContainsWhitespace)
        }
    }

    pub fn is_valid_name(name: &str) -> bool {
        !name.chars().any(|c| c.is_whitespace())
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
mod tests {
    use super::*;

    #[test]
    fn new() {
        let label = Label::new("label");
        assert!(label.is_ok());

        let label = Label::new("label 2 electric boogaloo");
        assert!(label.is_err());
    }
}
