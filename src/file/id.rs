use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct FileId {
    data: u64,
}

impl FileId {
    pub fn new(data: u64) -> Self {
        Self { data }
    }

    pub fn value(&self) -> u64 {
        self.data
    }
}

impl From<FileId> for String {
    fn from(id: FileId) -> Self {
        id.data.to_string()
    }
}

impl TryFrom<String> for FileId {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let data = s.parse::<u64>()?;
        Ok(FileId { data })
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VALUE: u64 = 12345;
    const TEST_STRING: &str = "12345";
    const INVALID_STRING: &str = "not_a_number";

    #[test]
    fn test_new_stores_value() {
        let id = FileId::new(TEST_VALUE);

        assert_eq!(id.data, TEST_VALUE);
    }

    #[test]
    fn test_value_returns_data() {
        let id = FileId::new(TEST_VALUE);

        assert_eq!(id.value(), TEST_VALUE);
    }

    #[test]
    fn test_into_string_converts_correctly() {
        let id = FileId::new(TEST_VALUE);

        let string_representation = String::from(id);

        assert_eq!(string_representation, TEST_STRING);
    }

    #[test]
    fn test_try_from_valid_string_succeeds() {
        let result = FileId::try_from(TEST_STRING.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), TEST_VALUE);
    }

    #[test]
    fn test_try_from_invalid_string_fails() {
        let result = FileId::try_from(INVALID_STRING.to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_display_formats_correctly() {
        let id = FileId::new(TEST_VALUE);

        assert_eq!(format!("{}", id), TEST_STRING);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let id = FileId::new(TEST_VALUE);

        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: FileId = serde_json::from_str(&serialized).unwrap();

        assert_eq!(id, deserialized);
    }
}
