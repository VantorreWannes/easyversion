use std::{
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::project::Project;

#[derive(Debug, PartialEq, Default, Eq, Clone, Serialize, Deserialize)]
pub struct EasyVersion {
    projects: HashMap<PathBuf, Vec<Project>>,
    current_project: Option<(PathBuf, usize)>,
}

impl EasyVersion {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            current_project: None,
        }
    }

    pub fn new_project(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let project = Project::new(path)?;
        if let Some(projects) = self.projects.get_mut(path) {
            projects.push(project);
        } else {
            self.projects.insert(path.to_path_buf(), vec![project]);
        }
        self.set_current_project(path, self.projects[path].len() - 1);
        Ok(())
    }

    pub fn save_projects(&self) -> io::Result<()> {
        for key in self.projects.keys() {
            let hash = hash(key);
            let path = Self::config_path().join(hash.to_string());
            fs::create_dir_all(path.clone())?;
            let projects = self
                .projects
                .get(key)
                .expect("key should exist and contain projects");
            for (index, project) in projects.iter().enumerate() {
                let mut path = path.clone();
                path.push(format!("ezproject_{}.ron", index));
                project.write_to(path)?;
            }
        }
        Ok(())
    }

    pub fn open_projects(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let hash = hash(path.as_ref().to_path_buf());
        let path = Self::config_path().join(hash.to_string());
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let project = Project::open_from(entry.path())?;
            if let Some(projects) = self.projects.get_mut(&path) {
                projects.push(project);
            } else {
                self.projects.insert(path.to_path_buf(), vec![project]);
            }
        }
        Ok(())
    }

    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir().expect("config dir should exist");
        config_dir.join("easyversion")
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        if let Some((path, index)) = &self.current_project {
            let project = self.projects.get(path)?;
            return project.get(*index);
        }
        None
    }

    pub fn get_current_project_mut(&mut self) -> Option<&mut Project> {
        if let Some((path, index)) = &self.current_project {
            let project = self.projects.get_mut(path)?;
            return project.get_mut(*index);
        }
        None
    }

    pub fn set_current_project(&mut self, path: impl AsRef<Path>, index: usize) {
        self.current_project = Some((path.as_ref().to_path_buf(), index));
    }

    pub fn save(&self) -> io::Result<()> { 
        let easy_version_string = ron::to_string(self).expect("serializing should succeed");
        fs::write(Self::config_path().join("easy_version.ron"), easy_version_string)
    }

    pub fn load() -> io::Result<Self> {
        let easy_version_string = fs::read_to_string(Self::config_path().join("easy_version.ron"))?;
        Ok(ron::from_str(&easy_version_string).expect("deserializing should succeed"))
    }
}

pub fn hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod easy_version_tests {
    use std::io;

    use super::EasyVersion;

    #[test]
    fn new() {
        let _ = EasyVersion::new();
    }

    #[test]
    fn new_project() -> io::Result<()> {
        let mut easy_version = EasyVersion::default();
        easy_version.new_project("test-data/easy-version/items/A")?;
        assert!(easy_version.current_project.is_some());
        Ok(())
    }

    #[test]
    fn save_projects() -> io::Result<()> {
        let mut easy_version = EasyVersion::default();
        easy_version.new_project("test-data/easy-version/items/A")?;
        easy_version.save_projects()?;
        Ok(())
    }

    #[test]
    fn open_projects() -> io::Result<()> {
        let mut easy_version = EasyVersion::default();
        easy_version.new_project("test-data/easy-version/items/A")?;
        easy_version.save_projects()?;
        easy_version.open_projects("test-data/easy-version/items/A")?;
        assert!(easy_version.current_project.is_some());
        Ok(())
    }
}
