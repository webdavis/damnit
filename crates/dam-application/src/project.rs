mod find;

use std::path::PathBuf;

pub use find::{ProjectFolders, find_project};

pub const PROJECT_FILE: &str = ".dam.lua";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSettings {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    pub folder: PathBuf,
    pub settings: ProjectSettings,
}

impl Project {
    pub fn file(&self) -> PathBuf {
        self.folder.join(PROJECT_FILE)
    }
}

#[cfg(test)]
mod tests;
