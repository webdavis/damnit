use dam_application::{HookFile, HookSetting};
use serde::Serialize;

use crate::error::ConfigReason;

#[derive(Serialize)]
pub(super) struct HookRow {
    pub(super) hook: Option<&'static str>,
    pub(super) file: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(super) project: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(super) off: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) changed: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) loads: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) on_failure: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) changes: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<String>,
    #[serde(skip)]
    pub(super) by_default: bool,
}

impl HookRow {
    pub(super) fn named(file: String) -> HookRow {
        HookRow {
            hook: None,
            file,
            project: true,
            off: false,
            changed: None,
            loads: None,
            on_failure: None,
            changes: None,
            error: None,
            by_default: false,
        }
    }

    pub(super) fn of(hook: &HookFile) -> HookRow {
        let mut row = HookRow {
            hook: hook.event.map(|event| event.as_str()),
            project: false,
            loads: Some(hook.read.is_ok()),
            ..HookRow::named(hook.file.clone())
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

    pub(super) fn cells(&self) -> Vec<String> {
        let hook = self.hook.unwrap_or_default().to_string();
        let mut cells = vec![hook, self.file.clone()];
        if self.off {
            cells.push("off".into());
            return cells;
        }
        if let Some(changed) = self.changed {
            cells.push(changed.into());
            return cells;
        }
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
