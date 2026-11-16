mod find;

use std::path::PathBuf;

use crate::TaskTemplate;

pub use find::{ProjectFolders, find_project};

pub const PROJECT_FILE: &str = ".dam.lua";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectSettings {
    pub path: String,
    pub new_task: TaskTemplate,
    pub required: Vec<RequiredField>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredField {
    Due,
    Labels,
    Priority,
    Body,
    Deadline,
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
