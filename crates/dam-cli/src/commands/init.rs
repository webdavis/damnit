use std::path::Path as FsPath;

use dam_adapters::write_project_file;
use dam_application::{NewTask, PROJECT_FILE, ProjectSettings};
use dam_domain::{Object, Path};
use serde::Serialize;

use crate::args::InitArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

pub(crate) struct Here<'a> {
    pub(crate) folder: &'a FsPath,
    pub(crate) personal_config: &'a FsPath,
}

#[derive(Serialize)]
struct Initialized {
    file: String,
    path: Option<String>,
    created: bool,
    task: Option<String>,
    tasks_already_there: Option<usize>,
}

impl View for Initialized {
    fn human(&self) -> Human {
        let text = match (&self.path, &self.task, self.tasks_already_there) {
            (None, ..) => format!(
                "{} already makes this folder a project; nothing changed",
                self.file
            ),
            (Some(path), Some(task), _) => {
                format!("wrote {} with path {path}\ncreated task {task}", self.file)
            }
            (Some(path), None, Some(there)) => format!(
                "wrote {file} with path {path}\n{path} was already there, with {there} {tasks} under it; this folder now shares that place. To use another place, change path in {file}",
                file = self.file,
                tasks = if there == 1 { "task" } else { "tasks" },
            ),
            (Some(path), None, None) => format!("wrote {} with path {path}", self.file),
        };
        Human::Lines(text.lines().map(String::from).collect())
    }
}

pub(crate) fn run(ctx: &Context, args: InitArgs, here: Here) -> Result<Report, CliError> {
    let file = here.folder.join(PROJECT_FILE);
    if file.is_file() {
        return report(&Initialized {
            file: file.display().to_string(),
            path: None,
            created: false,
            task: None,
            tasks_already_there: None,
        });
    }
    if here.folder == here.personal_config {
        return Err(CliError::Usage(format!(
            "{} holds your own config, which is never a project; run dam init in a project's folder",
            here.folder.display()
        )));
    }
    let path = match args.path {
        Some(path) if path != Path::root() => path,
        _ => named_after(here.folder)?,
    };
    let tasks_already_there = under_a_held_path(ctx, &path)?;
    let task = match tasks_already_there {
        Some(_) => None,
        None => Some(created(ctx, &path)?),
    };
    let settings = ProjectSettings {
        path: path.as_str().to_string(),
    };
    let file = write_project_file(here.folder, &settings)?;
    report(&Initialized {
        file: file.display().to_string(),
        path: Some(settings.path),
        created: true,
        task,
        tasks_already_there,
    })
}

fn named_after(folder: &FsPath) -> Result<Path, CliError> {
    folder
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| Path::root().join(name).ok())
        .ok_or_else(|| {
            CliError::Usage(format!(
                "{} has no name dam can use as a path; name one with --path",
                folder.display()
            ))
        })
}

fn under_a_held_path(ctx: &Context, path: &Path) -> Result<Option<usize>, CliError> {
    let all = ctx.store.all()?;
    let bases = || all.iter().map(Object::base);
    let held = bases().any(|base| base.owned_folder().as_ref() == Some(path));
    let under = bases()
        .filter(|base| base.path.as_str().starts_with(path.as_str()))
        .count();
    Ok(held.then_some(under))
}

fn created(ctx: &Context, path: &Path) -> Result<String, CliError> {
    let parent = path.parent().unwrap_or_else(Path::root);
    let subject = path
        .as_str()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string();
    let landed = ctx.dam().new_task(NewTask {
        parent: None,
        subject,
        path: parent,
        priority: Default::default(),
        due: None,
        deadline: None,
        labels: Default::default(),
        body: String::new(),
        id: None,
    })?;
    Ok(landed.oid().short().to_string())
}

#[cfg(test)]
mod tests;
