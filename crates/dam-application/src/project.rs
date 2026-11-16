mod find;
mod hook_fingerprint;

use std::path::PathBuf;

use dam_domain::{Categories, ChildDisposition};

use crate::{FilterConfig, TaskTemplate};

pub use find::{ProjectFolders, find_project};
pub use hook_fingerprint::{HookEntry, HookFileChange, HookFingerprint};

pub const PROJECT_FILE: &str = ".dam.lua";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectSettings {
    pub path: String,
    pub minimum_dam: Option<String>,
    pub new_task: TaskTemplate,
    pub required: Vec<RequiredField>,
    pub ls: Option<String>,
    pub filters: Vec<FilterConfig>,
    pub categories: Categories,
    pub sections: Vec<String>,
    pub branch_label: bool,
    pub done_children: Option<ChildDisposition>,
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
        let filter = query.as_deref().and_then(|name| self.filter_query(name));
        filter.map(String::from).or(query)
    }

    pub fn filter_query(&self, name: &str) -> Option<&str> {
        self.filters
            .iter()
            .find(|filter| filter.name == name)
            .and_then(|filter| filter.rule.query())
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
