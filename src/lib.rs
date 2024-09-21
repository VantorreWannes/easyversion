use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

pub mod patch;
pub mod timeline;
pub mod tracked;

pub fn hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
pub mod test_tools {
    use std::{fs, path::PathBuf};

    pub fn dir_path(dirs: &[&str]) -> PathBuf {
        let mut test_dir_path = dirs::cache_dir().unwrap().join("easyversion").join("tests");
        test_dir_path.extend(dirs);
        fs::create_dir_all(&test_dir_path).expect("Testing shouldn't fail.");
        test_dir_path
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn testing() {}
}
