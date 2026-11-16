use std::path::Path as FsPath;

use dam_adapters::read_project_file;
use dam_application::ConfigError;
use dam_domain::Path;
use serde::Serialize;

use super::{Initialized, created, under_a_held_path};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

#[derive(Serialize)]
pub(super) struct SectionCreated {
    pub(super) name: String,
    pub(super) task: String,
}

pub(super) fn completed(ctx: &Context, file: &FsPath) -> Result<Report, CliError> {
    let settings = read_project_file(file, env!("CARGO_PKG_VERSION"))?;
    let path = Path::parse(&settings.path).map_err(|_| ConfigError::NotAPath {
        file: file.to_path_buf(),
    })?;
    let task = match under_a_held_path(ctx, &path)? {
        Some(_) => None,
        None => Some(created(ctx, &path)?),
    };
    let mut sections_created = Vec::new();
    for name in settings.sections {
        let section = path.join(&name).map_err(|_| ConfigError::NotAPath {
            file: file.to_path_buf(),
        })?;
        if under_a_held_path(ctx, &section)?.is_none() {
            let task = created(ctx, &section)?;
            sections_created.push(SectionCreated { name, task });
        }
    }
    let changed = task.is_some() || !sections_created.is_empty();
    report(&Initialized {
        file: file.display().to_string(),
        path: changed.then_some(settings.path),
        created: false,
        task,
        tasks_already_there: None,
        sections_created,
    })
}
