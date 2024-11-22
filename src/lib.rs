use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

pub mod patches;
pub mod tracked;
pub mod version_info_manager;

pub fn hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
