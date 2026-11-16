use dam_adapters::{LuaConfigFile, ProjectHookFolder};
use dam_application::{Config, ConfigFile, HookFileChange, HookFingerprint, ProjectHookSwitches};

use super::HookRow;
use crate::error::CliError;
use crate::project_hooks::{CHANGED, ProjectHookAnswer, TOO_LARGE, answer};
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
    match answer(look.switches, &project.folder, &personal)? {
        ProjectHookAnswer::On(folder) => {
            let (files, unshown) = file.project_hook_files(&folder)?;
            let rows = files.iter().map(HookRow::of).map(|row| HookRow {
                project: true,
                ..row
            });
            Ok(ProjectRows {
                rows: rows.collect(),
                advice: None,
                unshown,
            })
        }
        ProjectHookAnswer::Off => {
            let (names, unshown) = ProjectHookFolder::read(&project.folder)?.names();
            let advice = (!names.is_empty()).then_some(TURN_ON);
            let rows = names.into_iter().map(|(file, event)| HookRow {
                hook: event.map(|event| event.as_str()),
                off: true,
                ..HookRow::named(file)
            });
            Ok(ProjectRows {
                rows: rows.collect(),
                advice,
                unshown,
            })
        }
        ProjectHookAnswer::Changed { folder, now, then } => {
            Ok(changed_rows(&folder, &now, then.as_ref()))
        }
        ProjectHookAnswer::TooLarge => Ok(ProjectRows {
            advice: Some(TOO_LARGE),
            ..ProjectRows::default()
        }),
    }
}

fn changed_rows(
    folder: &ProjectHookFolder,
    now: &HookFingerprint,
    then: Option<&HookFingerprint>,
) -> ProjectRows {
    let (names, unshown) = folder.names();
    let changes = then.map(|then| then.changes(now));
    let change_of = |file: &str| match changes.as_ref().map(|c| c.get(file)) {
        Some(Some(HookFileChange::New)) => "new",
        Some(None) => "unchanged",
        _ => "changed",
    };
    let mut rows: Vec<HookRow> = names
        .iter()
        .map(|(file, event)| HookRow {
            hook: event.map(|event| event.as_str()),
            changed: Some(change_of(file)),
            ..HookRow::named(file.clone())
        })
        .collect();
    let others = changes
        .iter()
        .flatten()
        .filter(|(file, _)| !names.iter().any(|(hook_file, _)| hook_file == *file))
        .map(|(file, change)| HookRow {
            changed: Some(match change {
                HookFileChange::Changed => "changed",
                HookFileChange::New => "new",
                HookFileChange::Removed => "removed",
            }),
            ..HookRow::named(file.clone())
        });
    rows.extend(others);
    ProjectRows {
        rows,
        advice: Some(CHANGED),
        unshown,
    }
}

#[derive(Default)]
pub(super) struct ProjectRows {
    pub(super) rows: Vec<HookRow>,
    pub(super) advice: Option<&'static str>,
    pub(super) unshown: usize,
}

pub(super) fn unshown_text(unshown: usize) -> Option<String> {
    match unshown {
        0 => None,
        1 => Some(
            "1 file in .damhooks is a link or has a name dam will not show; dam does not load it"
                .into(),
        ),
        n => Some(format!(
            "{n} files in .damhooks are links or have names dam will not show; dam does not load them"
        )),
    }
}
