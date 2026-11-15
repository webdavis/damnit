use dam_application::{
    Expected, HookConfig, HookEvent, Misread, OnFailure, PulledChanges, PulledHook,
};
use mlua::Value;

use super::strict::StrictMap;

pub(crate) const HOOK: [&str; 4] = event_names();
pub(crate) const HOOK_TABLE: [&str; 2] = ["run", "on_failure"];
pub(crate) const PULLED_HOOK_TABLE: [&str; 2] = ["run", "changes"];

const PULLED: &str = HookEvent::TaskPulled.as_str();

const fn event_names() -> [&'static str; 4] {
    let [new, changed, done] = HookEvent::LOCAL;
    [new.as_str(), changed.as_str(), done.as_str(), PULLED]
}

pub(super) type Hooks = (Vec<HookConfig>, Option<PulledHook>);

pub(super) fn read_hooks(root: &StrictMap) -> Result<Hooks, Misread> {
    let Some(hooks) = root.map("hook")? else {
        return Ok((Vec::new(), None));
    };
    hooks.only(&HOOK)?;
    let local = HookEvent::LOCAL
        .into_iter()
        .filter_map(|event| {
            let value = hooks.get(event.as_str())?;
            Some(read_hook(&hooks, event, value))
        })
        .collect::<Result<_, _>>()?;
    let pulled = hooks
        .get(PULLED)
        .map(|value| read_pulled_hook(&hooks, value))
        .transpose()?;
    Ok((local, pulled))
}

fn read_hook(hooks: &StrictMap, event: HookEvent, value: &Value) -> Result<HookConfig, Misread> {
    let on_failure = match value {
        Value::Function(_) => None,
        Value::Table(_) => {
            let hook = hooks.child(event.as_str(), value)?;
            hook.only(&HOOK_TABLE)?;
            read_run(&hook)?;
            read_on_failure(&hook)?
        }
        _ => return Err(hooks.wrong(event.as_str(), Expected::FunctionOrTable)),
    };
    Ok(HookConfig { event, on_failure })
}

fn read_pulled_hook(hooks: &StrictMap, value: &Value) -> Result<PulledHook, Misread> {
    let changes = match value {
        Value::Function(_) => None,
        Value::Table(_) => {
            let hook = hooks.child(PULLED, value)?;
            if hook.get("on_failure").is_some() {
                return Err(Misread::AlwaysWarns {
                    key: "on_failure".to_string(),
                    at: hook.place(),
                });
            }
            hook.only(&PULLED_HOOK_TABLE)?;
            read_run(&hook)?;
            read_changes(&hook)?
        }
        _ => return Err(hooks.wrong(PULLED, Expected::FunctionOrTable)),
    };
    Ok(PulledHook { changes })
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

fn read_run(hook: &StrictMap) -> Result<(), Misread> {
    match hook.get("run") {
        Some(Value::Function(_)) => Ok(()),
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
