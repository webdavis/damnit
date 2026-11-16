use std::path::PathBuf;

use dam_application::{ConfigError, TaskTemplate, TaskTemplates, TemplateName, TemplateOwner};
use tempfile::TempDir;

use crate::lua_config::FileSystemTemplates;

pub(super) struct Folders {
    pub(super) project: TempDir,
    pub(super) config: TempDir,
}

impl Folders {
    pub(super) fn new() -> Folders {
        Folders {
            project: tempfile::tempdir().unwrap(),
            config: tempfile::tempdir().unwrap(),
        }
    }

    pub(super) fn project_template(self, name: &str, text: &str) -> Folders {
        write(self.project.path().join(".damtemplates"), name, text);
        self
    }

    pub(super) fn own_template(self, name: &str, text: &str) -> Folders {
        write(self.config.path().join("templates"), name, text);
        self
    }

    pub(super) fn project_file(&self, name: &str) -> PathBuf {
        self.project
            .path()
            .join(".damtemplates")
            .join(format!("{name}.lua"))
    }

    pub(super) fn inside(&self) -> FileSystemTemplates {
        FileSystemTemplates::new(Some(self.project.path().into()), self.config.path().into())
    }

    pub(super) fn outside(&self) -> FileSystemTemplates {
        FileSystemTemplates::new(None, self.config.path().into())
    }
}

fn write(folder: PathBuf, name: &str, text: &str) {
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join(format!("{name}.lua")), text).unwrap();
}

pub(super) fn found(
    templates: &FileSystemTemplates,
    name: &str,
) -> Result<Option<TaskTemplate>, ConfigError> {
    templates.find(&TemplateName::parse(name).unwrap())
}

pub(super) fn names(templates: &FileSystemTemplates) -> Vec<(String, TemplateOwner)> {
    templates.names().unwrap()
}
