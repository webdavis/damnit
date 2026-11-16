use dam_adapters::LuaConfigFile;
use serde::Serialize;

use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

mod project;
mod row;

pub(crate) use project::ProjectHookLook;
use row::HookRow;

#[derive(Serialize)]
struct HookList {
    hooks: Vec<HookRow>,
    #[serde(skip)]
    advice: Option<&'static str>,
    #[serde(rename = "unshown_files", skip_serializing_if = "is_zero")]
    unshown: usize,
}

fn is_zero(count: &usize) -> bool {
    *count == 0
}

impl View for HookList {
    fn human(&self) -> Human {
        Human::list(
            self.hooks.iter().map(HookRow::cells).collect(),
            "no hooks in the hooks folder",
        )
    }

    fn warnings(&self) -> Vec<String> {
        let advice = self.advice.map(str::to_string);
        let errors = self.hooks.iter().filter_map(|hook| hook.error.clone());
        let unshown = project::unshown_text(self.unshown);
        errors.chain(unshown).chain(advice).collect()
    }
}

pub(crate) fn run(file: &LuaConfigFile, look: &ProjectHookLook) -> Result<Report, CliError> {
    let mut hooks: Vec<HookRow> = file.hook_files()?.iter().map(HookRow::of).collect();
    let theirs = project::rows(file, look)?;
    hooks.extend(theirs.rows);
    report(&HookList {
        hooks,
        advice: theirs.advice,
        unshown: theirs.unshown,
    })
}

#[cfg(test)]
fn listed(hooks: Vec<dam_application::HookFile>) -> Result<Report, CliError> {
    report(&HookList {
        hooks: hooks.iter().map(HookRow::of).collect(),
        advice: None,
        unshown: 0,
    })
}

#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod unshown_tests;
