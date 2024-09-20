use std::io;
pub mod file;
pub mod folder;

pub trait Version {
    fn save(&mut self) -> io::Result<()>;

    fn load(&mut self, index: usize) -> io::Result<()>;

    fn delete(&mut self, index: usize) -> io::Result<()>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn load_last(&mut self) -> io::Result<()> {
        self.load(self.len().saturating_sub(1))
    }

    fn delete_last(&mut self) -> io::Result<()> {
        self.delete(self.len().saturating_sub(1))
    }

    fn restore(&mut self) -> io::Result<()> {
        self.load_last()
    }

    fn clear(&mut self) -> io::Result<()> {
        self.delete(0)
    }

    fn split(&mut self, index: usize) -> io::Result<Self>
    where
        Self: Sized + Clone,
    {
        self.load(index)?;
        let mut other = self.clone();
        other.clear()?;
        other.save()?;
        Ok(other)
    }
}

#[cfg(test)]
mod version_test_tools {
    use std::{fs, io, path::PathBuf};

    pub fn dir_path(dirs: &[&str]) -> io::Result<PathBuf> {
        let mut patch_dir = dirs::config_dir()
            .unwrap()
            .join("easyversion")
            .join("tests");
        for dir in dirs {
            patch_dir = patch_dir.join(dir);
        }
        fs::create_dir_all(&patch_dir)?;
        Ok(patch_dir)
    }

    pub fn patch_dir_path(dirs: &[&str]) -> io::Result<PathBuf> {
        let test_dir = dir_path(dirs)?;
        let patch_dir = test_dir.join("patches");
        fs::create_dir_all(&patch_dir)?;
        Ok(patch_dir)
    }

    pub fn tracked_file_path(dirs: &[&str]) -> io::Result<PathBuf> {
        let test_dir = dir_path(dirs)?;
        let file_path = test_dir.join("file.txt");
        fs::write(&file_path, b"123")?;
        Ok(file_path)
    }

    pub fn tracked_folder_path(dirs: &[&str]) -> io::Result<PathBuf> {
        let test_dir = dir_path(dirs)?;
        let tracked_dir = test_dir.join("tracked_folder");
        fs::create_dir_all(&tracked_dir)?;
        let tracked_dir_subfolder = tracked_dir.join("subfolder");
        fs::create_dir_all(&tracked_dir_subfolder)?;
        fs::write(tracked_dir.join("file.txt"), b"123")?;
        fs::write(tracked_dir_subfolder.join("file.txt"), b"123")?;
        Ok(tracked_dir)
    }

    #[test]
    fn test_dir_path() -> io::Result<()> {
        let test_dir = dir_path(&["version_test_tools", "dir_path"])?;
        assert!(test_dir.exists());
        Ok(())
    }

    #[test]
    fn test_patch_dir_path() -> io::Result<()> {
        let test_dir = patch_dir_path(&["version_test_tools", "patch_dir_path"])?;
        assert!(test_dir.exists());
        Ok(())
    }

    #[test]
    fn test_tracked_file_path() -> io::Result<()> {
        let test_dir = tracked_file_path(&["version_test_tools", "tracked_file_path"])?;
        assert!(test_dir.exists());
        Ok(())
    }

    #[test]
    fn test_tracked_folder_path() -> io::Result<()> {
        let test_dir = tracked_folder_path(&["version_test_tools", "tracked_folder_path"])?;
        assert!(test_dir.exists());
        Ok(())
    }
}
