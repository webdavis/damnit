use std::path::Path as FsPath;

use dam_adapters::write_project_file;
use dam_application::{NewTask, PROJECT_FILE, ProjectSettings};
use dam_domain::{Object, Path};
use serde::Serialize;

use crate::args::InitArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::RunFolders;
use crate::view::report;

pub(crate) struct Here<'a> {
    pub(crate) folder: &'a FsPath,
    pub(crate) personal_config: &'a FsPath,
}

impl<'a> From<&'a RunFolders> for Here<'a> {
    fn from(run: &'a RunFolders) -> Here<'a> {
        Here {
            folder: &run.folder,
            personal_config: &run.personal_config,
        }
    }
}

#[derive(Serialize)]
struct Initialized {
    file: String,
    path: Option<String>,
    created: bool,
    task: Option<String>,
    tasks_already_there: Option<usize>,
    sections_created: Vec<completed::SectionCreated>,
}

pub(crate) fn run(ctx: &Context, args: InitArgs, here: Here) -> Result<Report, CliError> {
    let file = here.folder.join(PROJECT_FILE);
    if file.is_file() {
        return completed::completed(ctx, &file);
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
        ..Default::default()
    };
    let file = write_project_file(here.folder, &settings)?;
    report(&Initialized {
        file: file.display().to_string(),
        path: Some(settings.path),
        created: true,
        task,
        tasks_already_there,
        sections_created: Vec::new(),
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

mod completed;
mod told;

#[cfg(test)]
mod tests;
