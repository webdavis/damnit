use dam_adapters::LuaConfigFile;
use dam_application::{HookFile, HookSetting};
use serde::Serialize;

use crate::error::{CliError, ConfigReason};
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct HookList {
    hooks: Vec<HookRow>,
}

#[derive(Serialize)]
struct HookRow {
    hook: Option<&'static str>,
    file: String,
    loads: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    on_failure: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip)]
    by_default: bool,
}

impl HookRow {
    fn of(hook: &HookFile) -> HookRow {
        let mut row = HookRow {
            hook: hook.event.map(|event| event.as_str()),
            file: hook.file.clone(),
            loads: hook.read.is_ok(),
            on_failure: None,
            changes: None,
            error: None,
            by_default: false,
        };
        match &hook.read {
            Ok(HookSetting::OnFailure(written)) => {
                row.on_failure = Some(written.unwrap_or_default().as_str());
                row.by_default = written.is_none();
            }
            Ok(HookSetting::Changes(written)) => {
                row.changes = Some(written.unwrap_or_default().as_str());
                row.by_default = written.is_none();
            }
            Err(error) => row.error = Some(ConfigReason(error).to_string()),
        }
        row
    }

    fn cells(&self) -> Vec<String> {
        let hook = self.hook.unwrap_or_default().to_string();
        let mut cells = vec![hook, self.file.clone()];
        match (self.on_failure, self.changes) {
            (Some(value), _) => cells.extend(["on_failure".into(), self.marked(value)]),
            (_, Some(value)) => cells.extend(["changes".into(), self.marked(value)]),
            (None, None) => cells.push("does not load".into()),
        }
        cells
    }

    fn marked(&self, value: &str) -> String {
        match self.by_default {
            true => format!("{value} (default)"),
            false => value.to_string(),
        }
    }
}

impl View for HookList {
    fn human(&self) -> Human {
        Human::list(
            self.hooks.iter().map(HookRow::cells).collect(),
            "no hooks in the hooks folder",
        )
    }

    fn warnings(&self) -> Vec<String> {
        self.hooks
            .iter()
            .filter_map(|hook| hook.error.clone())
            .collect()
    }
}

pub(crate) fn run(file: &LuaConfigFile) -> Result<Report, CliError> {
    listed(file.hook_files()?)
}

fn listed(hooks: Vec<HookFile>) -> Result<Report, CliError> {
    report(&HookList {
        hooks: hooks.iter().map(HookRow::of).collect(),
    })
}

#[cfg(test)]
mod tests;
