use dam_domain::{Path, Priority};

use super::ConfigError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskTemplate {
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub priority: Option<Priority>,
    pub due: Option<String>,
    pub deadline: Option<String>,
    pub path: Option<Path>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateOwner {
    Project,
    Person,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateName(String);

#[derive(Debug, PartialEq, Eq)]
pub struct NotATemplateName;

impl TemplateName {
    pub fn parse(name: &str) -> Result<TemplateName, NotATemplateName> {
        let leaves_the_folder = name.contains(['/', '\\']) || name.contains("..");
        if name.is_empty() || leaves_the_folder || name.chars().any(char::is_control) {
            return Err(NotATemplateName);
        }
        Ok(TemplateName(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn file_name(&self) -> String {
        format!("{}.lua", self.0)
    }
}

pub trait TaskTemplates {
    fn find(&self, name: &TemplateName) -> Result<Option<TaskTemplate>, ConfigError>;
    fn names(&self) -> Result<Vec<(String, TemplateOwner)>, ConfigError>;
}

#[cfg(test)]
mod tests;
