mod find;

use std::path::PathBuf;

use crate::{FilterConfig, TaskTemplate};

pub use find::{ProjectFolders, find_project};

pub const PROJECT_FILE: &str = ".dam.lua";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectSettings {
    pub path: String,
    pub minimum_dam: Option<String>,
    pub new_task: TaskTemplate,
    pub required: Vec<RequiredField>,
    pub ls: Option<String>,
    pub filters: Vec<FilterConfig>,
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

impl ProjectSettings {
    pub fn listed(&self, query: Option<String>) -> Option<String> {
        self.filtered(query.or_else(|| self.ls.clone()))
    }

    pub fn filtered(&self, query: Option<String>) -> Option<String> {
        let filter = query.as_deref().and_then(|name| {
            self.filters
                .iter()
                .find(|filter| filter.name == name)
                .and_then(|filter| filter.rule.query())
        });
        filter.map(String::from).or(query)
    }
}

impl RequiredField {
    pub fn name(self) -> &'static str {
        match self {
            RequiredField::Due => "due",
            RequiredField::Labels => "labels",
            RequiredField::Priority => "priority",
            RequiredField::Body => "body",
            RequiredField::Deadline => "deadline",
        }
    }
}

impl Project {
    pub fn file(&self) -> PathBuf {
        self.folder.join(PROJECT_FILE)
    }
}

#[cfg(test)]
mod tests;
