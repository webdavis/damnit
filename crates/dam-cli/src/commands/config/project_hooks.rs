use std::path::Path;

use dam_adapters::ProjectHookFolder;
use dam_application::{
    Config, ProjectHookSwitch, ProjectHookSwitches, SettingChange, SettingKey, SettingValue,
};

use super::set::{bad_value, changed, refused};
use super::setting::{Setting, Value};
use crate::error::CliError;
use crate::output::Report;
use crate::project_hooks::{ProjectHookAnswer, answer};
use crate::project_scope::Standing;

pub(super) const KEY: &str = "project.hooks";

const TOO_LARGE_TO_TURN_ON: &str = "this project's .damhooks folder holds more than dam checks, 1000 entries or 8 MiB, so dam cannot turn its hooks on";

pub(super) fn this_clone<'a>(key: &str, global: bool, standing: &'a Standing) -> Option<&'a Path> {
    match standing {
        Standing::Inside(project) if key == KEY && !global => Some(&project.folder),
        _ => None,
    }
}

pub(super) fn set(
    switches: &dyn ProjectHookSwitches,
    clone: &Path,
    value: Option<&str>,
) -> Result<Report, CliError> {
    let key = SettingKey::parse(KEY).map_err(refused)?;
    let on = match value.map(|text| key.value(text)) {
        Some(Ok(SettingValue::Flag(on))) => Some(on),
        Some(Ok(_)) => None,
        Some(Err(bad)) => return Err(bad_value(bad)),
        None => None,
    };
    let switch = match on {
        Some(true) => match ProjectHookFolder::read(clone)?.fingerprint()? {
            Some(fingerprint) => Some(ProjectHookSwitch::on_for(fingerprint)),
            None => return Err(CliError::Usage(TOO_LARGE_TO_TURN_ON.into())),
        },
        Some(false) => Some(ProjectHookSwitch::off()),
        None => None,
    };
    let old = switches.switch(clone)?;
    switches.turn(clone, switch)?;
    changed(
        &SettingChange {
            key: KEY.into(),
            old: old.map(|old| old.on.to_string()),
            new: on.map(|on| on.to_string()),
        },
        None,
    )
}

pub(super) enum From {
    ThisClone,
    ConfigLua,
    Default,
}

impl From {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            From::ThisClone => "this clone",
            From::ConfigLua => "config.lua",
            From::Default => "default",
        }
    }
}

pub(super) fn in_force(
    switches: &dyn ProjectHookSwitches,
    clone: &Path,
    config: &Config,
) -> Result<(ProjectHookAnswer, From), CliError> {
    let from = match (switches.switch(clone)?, config.project_hooks) {
        (Some(_), _) => From::ThisClone,
        (None, Some(_)) => From::ConfigLua,
        (None, None) => From::Default,
    };
    Ok((answer(switches, clone, config)?, from))
}

pub(super) fn personal(config: &Config) -> (String, Setting) {
    let on = Value::Flag(config.project_hooks_on(None));
    let hooks = match config.project_hooks {
        Some(_) => Setting::set(on),
        None => Setting::by_default(on),
    };
    (
        "project".into(),
        Setting::part(vec![("hooks".into(), hooks)]),
    )
}
