use std::path::{Path, PathBuf};

use dam_application::{ConfigError, TaskTemplate, TaskTemplates, TemplateName, TemplateOwner};

use super::super::evaluate::read_text;
use super::super::project::read_project_template;
use super::read_own_template;

const PROJECT_TEMPLATES: &str = ".damtemplates";
const OWN_TEMPLATES: &str = "templates";

pub struct FileSystemTemplates {
    project: Option<PathBuf>,
    own: PathBuf,
}

impl FileSystemTemplates {
    pub fn new(project_folder: Option<PathBuf>, config_folder: PathBuf) -> FileSystemTemplates {
        FileSystemTemplates {
            project: project_folder.map(|folder| folder.join(PROJECT_TEMPLATES)),
            own: config_folder.join(OWN_TEMPLATES),
        }
    }

    fn folders(&self) -> impl Iterator<Item = (&Path, TemplateOwner)> {
        let project = self.project.as_deref().map(|f| (f, TemplateOwner::Project));
        project
            .into_iter()
            .chain([(self.own.as_path(), TemplateOwner::Person)])
    }
}

impl TaskTemplates for FileSystemTemplates {
    fn find(&self, name: &TemplateName) -> Result<Option<TaskTemplate>, ConfigError> {
        for (folder, owner) in self.folders() {
            let file = folder.join(name.file_name());
            if !file.is_file() {
                continue;
            }
            let text = read_text(&file)?.unwrap_or_default();
            return match owner {
                TemplateOwner::Project => read_project_template(&file, &text),
                TemplateOwner::Person => read_own_template(&file, &text),
            }
            .map(Some);
        }
        Ok(None)
    }

    fn names(&self) -> Result<Vec<(String, TemplateOwner)>, ConfigError> {
        let mut names = Vec::new();
        for (folder, owner) in self.folders() {
            let mut held = names_in(folder)?;
            held.sort();
            names.extend(held.into_iter().map(|name| (name, owner)));
        }
        Ok(names)
    }
}

fn names_in(folder: &Path) -> Result<Vec<String>, ConfigError> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(ConfigError::Unreadable {
                path: folder.to_path_buf(),
                why: e.to_string(),
            });
        }
    };
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "lua"))
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_string()))
        .filter(|name| TemplateName::parse(name).is_ok())
        .collect())
}
