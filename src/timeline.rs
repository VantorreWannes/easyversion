use std::{
    io::{self},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::patch::Patch;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Timeline {
    patch_dir: PathBuf,
    patch_paths: Vec<PathBuf>,
}

impl Timeline {
    pub fn new(patch_dir: impl AsRef<Path>) -> Self {
        Self {
            patch_dir: patch_dir.as_ref().to_path_buf(),
            patch_paths: vec![],
        }
    }

    pub fn push(&mut self, patch: &Patch) -> io::Result<()> {
        let patch_path = self.patch_dir.join(format!("{}.bz2", patch.id()));
        let mut patch_file = std::fs::File::create(&patch_path)?;
        patch.write_to(&mut patch_file)?;
        self.patch_paths.push(patch_path);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<io::Result<Patch>> {
        if let Some(patch_path) = self.patch_paths.pop() {
            let mut patch_file = match std::fs::File::open(&patch_path) {
                Ok(patch_file) => patch_file,
                Err(err) => return Some(Err(err)),
            };
            return Some(Patch::read_from(&mut patch_file));
        }
        None
    }

    pub fn get(&self, index: usize) -> Option<io::Result<Patch>> {
        self.patch_paths.get(index).map(|path| {
            let mut patch_file = match std::fs::File::open(path) {
                Ok(patch_file) => patch_file,
                Err(err) => return Err(err),
            };
            Patch::read_from(&mut patch_file)
        })
    }

    pub fn len(&self) -> usize {
        self.patch_paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patch_paths.is_empty()
    }
}

#[cfg(test)]
mod timeline_tests {

    use super::*;

    fn patch_dir(name: &str) -> io::Result<PathBuf> {
        let patch_dir = dirs::config_dir()
            .unwrap()
            .join("easyversion")
            .join("tests")
            .join("timeline")
            .join(name);
        std::fs::create_dir_all(&patch_dir)?;
        Ok(patch_dir)
    }
    #[test]
    fn new() -> io::Result<()> {
        let patch_dir = patch_dir("new")?;
        let timeline = Timeline::new(&patch_dir);
        assert!(timeline.is_empty());
        assert_eq!(timeline.len(), 0);
        Ok(())
    }

    #[test]
    fn push() -> io::Result<()> {
        let patch_dir = patch_dir("push")?;
        let mut timeline = Timeline::new(&patch_dir);
        assert!(timeline.is_empty());
        let patch = Patch::from_data(&[2]);
        timeline.push(&patch)?;
        assert!(!timeline.is_empty());
        Ok(())
    }

    #[test]
    fn pop() -> io::Result<()> {
        let patch_dir = patch_dir("pop")?;
        let mut timeline = Timeline::new(&patch_dir);
        assert!(timeline.is_empty());
        let patch = Patch::from_data(&[2]);
        timeline.push(&patch)?;
        assert!(!timeline.is_empty());
        let popped_patch = timeline.pop();
        assert!(popped_patch.is_some());
        let popped_patch = popped_patch.unwrap()?;
        assert_eq!(popped_patch, patch);
        assert!(timeline.is_empty());
        Ok(())
    }

    #[test]
    fn get() -> io::Result<()> {
        let patch_dir = patch_dir("get")?;
        let mut timeline = Timeline::new(&patch_dir);
        assert!(timeline.is_empty());
        let patch = Patch::from_data(&[2]);
        timeline.push(&patch)?;
        assert!(!timeline.is_empty());
        let patch = timeline.get(0);
        assert!(patch.is_some());
        let patch = patch.unwrap()?;
        assert_eq!(patch, patch);
        Ok(())
    }
}
