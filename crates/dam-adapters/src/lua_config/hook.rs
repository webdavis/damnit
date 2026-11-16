use dam_application::{ConfigError, Expected, HookEvent, Misread, OnFailure, PulledChanges};
use mlua::{Function, Value};

use super::shape::described;
use super::strict::StrictMap;

pub(crate) const HOOK: [&str; 4] = event_names();
pub(crate) const HOOK_TABLE: [&str; 2] = ["run", "on_failure"];
pub(crate) const PULLED_HOOK_TABLE: [&str; 2] = ["run", "changes"];

const fn event_names() -> [&'static str; 4] {
    let [new, changed, done] = HookEvent::LOCAL;
    [
        new.as_str(),
        changed.as_str(),
        done.as_str(),
        HookEvent::TaskPulled.as_str(),
    ]
}

pub(super) fn event_named(name: &str) -> Option<HookEvent> {
    HookEvent::LOCAL
        .into_iter()
        .chain([HookEvent::TaskPulled])
        .find(|event| event.as_str() == name)
}

pub(super) struct ReadHook {
    pub(super) run: Function,
    pub(super) on_failure: Option<OnFailure>,
    pub(super) changes: Option<PulledChanges>,
}

pub(super) fn read_hook(
    file: &str,
    event: HookEvent,
    value: Value,
) -> Result<ReadHook, ConfigError> {
    let table = match value {
        Value::Function(run) => {
            return Ok(ReadHook {
                run,
                on_failure: None,
                changes: None,
            });
        }
        Value::Table(table) => table,
        other => {
            return Err(ConfigError::NotAHook {
                file: file.to_string(),
                returned: described(&other),
            });
        }
    };
    let hook = StrictMap::root(file, &table)?;
    if event != HookEvent::TaskPulled {
        hook.only(&HOOK_TABLE)?;
        return Ok(ReadHook {
            run: read_run(&hook)?,
            on_failure: read_on_failure(&hook)?,
            changes: None,
        });
    }
    if hook.get("on_failure").is_some() {
        return Err(Misread::AlwaysWarns {
            key: "on_failure".to_string(),
            at: hook.place(),
        }
        .into());
    }
    hook.only(&PULLED_HOOK_TABLE)?;
    Ok(ReadHook {
        run: read_run(&hook)?,
        on_failure: None,
        changes: read_changes(&hook)?,
    })
}

fn read_changes(hook: &StrictMap) -> Result<Option<PulledChanges>, Misread> {
    let Some(written) = hook.string("changes")? else {
        return Ok(None);
    };
    PulledChanges::ALL
        .into_iter()
        .find(|choice| choice.as_str() == written)
        .map(Some)
        .ok_or_else(|| hook.wrong("changes", Expected::CommittedOrWorking))
}

fn read_run(hook: &StrictMap) -> Result<Function, Misread> {
    match hook.get("run") {
        Some(Value::Function(run)) => Ok(run.clone()),
        Some(_) => Err(hook.wrong("run", Expected::Function)),
        None => Err(hook.missing("run")),
    }
}

fn read_on_failure(hook: &StrictMap) -> Result<Option<OnFailure>, Misread> {
    let Some(written) = hook.string("on_failure")? else {
        return Ok(None);
    };
    OnFailure::ALL
        .into_iter()
        .find(|choice| choice.as_str() == written)
        .map(Some)
        .ok_or_else(|| hook.wrong("on_failure", Expected::WarnOrRefuse))
}
