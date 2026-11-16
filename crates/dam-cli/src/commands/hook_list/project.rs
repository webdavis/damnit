use dam_adapters::LuaConfigFile;
use dam_application::{Config, ConfigFile, ProjectHookSwitches};

use super::HookRow;
use crate::error::CliError;
use crate::project_scope::Standing;

pub(super) const TURN_ON: &str =
    "this project's hooks are off; dam config set project.hooks true turns them on";

pub(crate) struct ProjectHookLook<'a> {
    pub(crate) standing: &'a Standing,
    pub(crate) switches: &'a dyn ProjectHookSwitches,
}

impl<'a> ProjectHookLook<'a> {
    pub(crate) fn of(
        standing: &'a Standing,
        switches: &'a dyn ProjectHookSwitches,
    ) -> ProjectHookLook<'a> {
        ProjectHookLook { standing, switches }
    }
}

pub(super) fn rows(file: &LuaConfigFile, look: &ProjectHookLook) -> Result<ProjectRows, CliError> {
    let Standing::Inside(project) = look.standing else {
        return Ok(ProjectRows::default());
    };
    let personal = Config {
        project_hooks: file.load().ok().and_then(|l| l.config.project_hooks),
        ..Config::default()
    };
    if personal.project_hooks_on(look.switches.switch(&project.folder)?) {
        let (files, unshown) = file.project_hook_files(&project.folder)?;
        let rows = files.iter().map(HookRow::of).map(|row| HookRow {
            project: true,
            ..row
        });
        return Ok(ProjectRows {
            rows: rows.collect(),
            off: false,
            unshown,
        });
    }
    let (names, unshown) = LuaConfigFile::project_hook_names(&project.folder)?;
    let off = !names.is_empty();
    let rows = names.into_iter().map(|(file, event)| HookRow {
        hook: event.map(|event| event.as_str()),
        file,
        project: true,
        off: true,
        loads: None,
        on_failure: None,
        changes: None,
        error: None,
        by_default: false,
    });
    Ok(ProjectRows {
        rows: rows.collect(),
        off,
        unshown,
    })
}

#[derive(Default)]
pub(super) struct ProjectRows {
    pub(super) rows: Vec<HookRow>,
    pub(super) off: bool,
    pub(super) unshown: usize,
}

pub(super) fn unshown_text(unshown: usize) -> Option<String> {
    match unshown {
        0 => None,
        1 => Some("1 file in .damhooks has a name dam will not show".into()),
        n => Some(format!(
            "{n} files in .damhooks have names dam will not show"
        )),
    }
}
