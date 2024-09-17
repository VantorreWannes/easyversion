use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

pub mod patch;
pub mod version;

pub fn hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {

    #[test]
    fn testing() {}
}
